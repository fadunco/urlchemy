use crate::{
    CloseReason, DesktopError, DesktopServices, HotkeyBridge, HotkeyLatch, Launcher,
    PaletteCommand, PaletteEffect, PaletteEvent, PaletteView,
};
use gpui_kit::{
    component::{ActiveTheme, button::Button},
    prelude::FluentBuilder,
    *,
};
use std::rc::Rc;
use urlchemy_core::{Catalog, Diagnostics, config};

actions!(urlchemy, [OpenLauncher, Quit]);

pub enum StartupFailure {
    Configuration(Diagnostics),
    Desktop(DesktopError),
}
pub enum StartupState {
    Loading,
    Ready(Rc<Catalog>),
    Failed(StartupFailure),
}
struct ShellGlobal(Entity<AppShell>);
impl Global for ShellGlobal {}

pub struct AppShell {
    startup: StartupState,
    services: Rc<dyn DesktopServices>,
    invocation: u64,
    window: Option<AnyWindowHandle>,
    palette: Option<Entity<PaletteView>>,
    registration_error: Option<String>,
    bridge: Option<HotkeyBridge>,
    loading: Option<Task<()>>,
    opening: Option<Task<()>>,
    subscriptions: Vec<Subscription>,
    quit_subscription: Option<Subscription>,
    stopped: bool,
    maximum_height: Pixels,
}
impl AppShell {
    pub fn install(
        initial: StartupState,
        services: Rc<dyn DesktopServices>,
        cx: &mut App,
    ) -> Entity<Self> {
        let shell = cx.new(|_| Self {
            startup: initial,
            services,
            invocation: 0,
            window: None,
            palette: None,
            registration_error: None,
            bridge: None,
            loading: None,
            opening: None,
            subscriptions: Vec::new(),
            quit_subscription: None,
            stopped: false,
            maximum_height: Pixels::ZERO,
        });
        cx.set_global(ShellGlobal(shell.clone()));
        cx.set_activation_policy(ActivationPolicy::Regular);
        cx.on_action(|_: &OpenLauncher, cx| {
            let shell = cx.global::<ShellGlobal>().0.clone();
            cx.defer(move |cx| shell.update(cx, |shell, cx| shell.invoke(cx)));
        });
        cx.on_action(|_: &Quit, cx| {
            let shell = cx.global::<ShellGlobal>().0.clone();
            cx.defer(move |cx| {
                shell.update(cx, |shell, cx| shell.shutdown(cx));
                cx.quit();
            });
        });
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus(vec![Menu::new("URLchemy").items([
            MenuItem::action("Open launcher…", OpenLauncher),
            MenuItem::Separator,
            MenuItem::action("Quit URLchemy", Quit),
        ])]);
        shell.update(cx, |shell, cx| {
            shell.quit_subscription = Some(cx.on_app_quit(|shell, cx| {
                shell.shutdown(cx);
                async {}
            }));
            if matches!(shell.startup, StartupState::Loading) {
                match shell.services.configuration_path() {
                    Ok(path) => {
                        let task = cx
                            .background_executor()
                            .spawn(async move { config::load_file(&path) });
                        shell.loading = Some(cx.spawn(async move |owner, cx| {
                            let result = task.await;
                            let _ = owner.update(cx, |shell, cx| {
                                if shell.stopped {
                                    return;
                                }
                                shell.startup = match result {
                                    Ok(catalog) => StartupState::Ready(Rc::new(catalog)),
                                    Err(diagnostics) => StartupState::Failed(
                                        StartupFailure::Configuration(diagnostics),
                                    ),
                                };
                                shell.register(cx);
                                if shell.window.is_some() {
                                    shell.invoke(cx);
                                }
                            });
                        }));
                    }
                    Err(error) => {
                        shell.startup = StartupState::Failed(StartupFailure::Desktop(error))
                    }
                }
            } else {
                shell.register(cx);
            }
            shell.invoke(cx);
        });
        shell
    }
    fn register(&mut self, cx: &mut Context<Self>) {
        let StartupState::Ready(catalog) = &self.startup else {
            return;
        };
        match self
            .services
            .register_invocation(catalog.shortcuts().invoke())
        {
            Ok(registration) => {
                let mut latch = HotkeyLatch::new(registration.id());
                let owner = cx.weak_entity();
                self.bridge = Some(HotkeyBridge::start(
                    registration.wait(),
                    move |signal, cx| {
                        if latch.accept(signal) {
                            let _ = owner.update(cx, |shell, cx| shell.invoke(cx));
                        }
                    },
                    cx,
                ));
            }
            Err(_) => {
                self.registration_error = Some(format!(
                    "Could not register {}. Use the Dock or Open launcher menu. Restart URLchemy after resolving the shortcut conflict.",
                    catalog.shortcuts().invoke().binding()
                ))
            }
        }
    }
    pub fn invoke(&mut self, cx: &mut Context<Self>) {
        if self.stopped {
            return;
        }
        let Some(invocation) = self.invocation.checked_add(1) else {
            return;
        };
        self.invocation = invocation;
        self.opening = None;
        self.remove_window(cx);
        let area = match self.services.work_area(cx) {
            Ok(area) => area,
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        };
        self.services.remember_frontmost(invocation);
        let owner = cx.weak_entity();
        let catalog = match &self.startup {
            StartupState::Ready(catalog) => Some(catalog.clone()),
            _ => None,
        };
        let status = self.startup_message();
        let banner = self.registration_error.clone();
        let rem = cx.theme().font_size;
        let bounds = palette_bounds(area, rem, rem * 30.);
        self.maximum_height = (area.bottom() - rem - bounds.top()).max(Pixels::ZERO);
        // The window boundary uses platform points. Descendants use rem-based layout.
        let opened = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                kind: WindowKind::PopUp,
                titlebar: None,
                is_resizable: false,
                focus: false,
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let palette = catalog.map(|catalog| {
                    cx.new(|cx| PaletteView::new(Launcher::new(catalog, invocation), window, cx))
                });
                cx.new(|cx| {
                    let focus = cx.focus_handle().tab_index(0);
                    if palette.is_none() {
                        focus.focus(window, cx);
                    }
                    ShellView {
                        palette,
                        status,
                        banner,
                        owner,
                        invocation,
                        focus,
                    }
                })
            },
        );
        let Ok((handle, frame)) = opened else {
            self.services.discard_frontmost(invocation);
            eprintln!("Could not create the launcher window.");
            return;
        };
        self.window = Some(handle);
        self.palette = frame.read(cx).palette.clone();
        if let Some(palette) = &self.palette {
            let subscription = cx.subscribe(palette, |_, _, event: &PaletteEvent, cx| {
                let event = event.clone();
                let owner = cx.weak_entity();
                cx.defer(move |cx| {
                    let _ = owner.update(cx, |shell, cx| match event {
                        PaletteEvent::Effect(effect) => shell.effect(effect, cx),
                        PaletteEvent::ContentHeight { invocation, height } => {
                            if invocation != shell.invocation {
                                return;
                            }
                            let Some(handle) = shell.window else {
                                return;
                            };
                            let maximum = shell.maximum_height;
                            let banner = shell.registration_error.is_some();
                            let _ = handle.update(cx, |_, window, _| {
                                // The banner has its own measured region outside the palette.
                                if banner {
                                    return;
                                }
                                let height = height.min(maximum);
                                let current = window.bounds().size;
                                if (current.height - height).abs() > px(0.5) {
                                    window.resize(size(current.width, height));
                                }
                            });
                        }
                    });
                });
            });
            self.subscriptions.push(subscription);
        }
        let owner = cx.weak_entity();
        let services = self.services.clone();
        let activation_owner = owner.clone();
        let activation = handle.update(cx, |_, window, cx| {
            let subscription = frame.update(cx, |_, cx| {
                cx.observe_window_activation(window, move |_, window, cx| {
                    if !window.is_window_active() && !services.is_launcher_frontmost() {
                        let owner = activation_owner.clone();
                        cx.defer(move |cx| {
                            let _ = owner.update(cx, |shell, cx| {
                                shell.close(invocation, CloseReason::Outside, cx)
                            });
                        });
                    }
                })
            });
            window.on_window_should_close(cx, move |_, cx| {
                let owner = owner.clone();
                cx.defer(move |cx| {
                    let _ = owner.update(cx, |shell, cx| {
                        shell.close(invocation, CloseReason::Explicit, cx)
                    });
                });
                false
            });
            cx.activate(true);
            window.activate_window();
            subscription
        });
        if let Ok(subscription) = activation {
            self.subscriptions.push(subscription);
        }
    }
    fn startup_message(&self) -> String {
        match &self.startup {
            StartupState::Loading => "Loading configuration…".into(),
            StartupState::Ready(_) => String::new(),
            StartupState::Failed(StartupFailure::Desktop(error)) => error.to_string(),
            StartupState::Failed(StartupFailure::Configuration(diagnostics)) => {
                let mut text = diagnostics
                    .items()
                    .iter()
                    .map(|diagnostic| {
                        let location = diagnostic
                            .location()
                            .map(|location| format!(":{}:{}", location.line, location.column))
                            .unwrap_or_default();
                        format!(
                            "{}{} {}: {}",
                            diagnostic.source().display(),
                            location,
                            diagnostic.pointer(),
                            diagnostic.message()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if diagnostics.truncated() {
                    text.push_str("\nAdditional diagnostics were omitted.");
                }
                text.push_str("\nCorrect the configuration file. Restart URLchemy.");
                text
            }
        }
    }
    fn effect(&mut self, effect: PaletteEffect, cx: &mut Context<Self>) {
        match effect {
            PaletteEffect::Close { invocation, reason } => self.close(invocation, reason, cx),
            PaletteEffect::Open {
                invocation,
                operation,
                url,
            } => {
                if invocation != self.invocation || self.window.is_none() || self.stopped {
                    return;
                }
                let task = self.services.open_url(&url, cx);
                self.opening = Some(cx.spawn(async move |owner, cx| {
                    let result = task.await;
                    let _ = owner.update(cx, |shell, cx| {
                        if shell.stopped || invocation != shell.invocation {
                            return;
                        }
                        if let (Some(handle), Some(palette)) = (shell.window, shell.palette.clone())
                        {
                            let _ = handle.update(cx, |_, window, cx| {
                                palette.update(cx, |palette, cx| {
                                    palette.dispatch(
                                        PaletteCommand::OpenFinished {
                                            invocation,
                                            operation,
                                            result,
                                        },
                                        window,
                                        cx,
                                    )
                                });
                            });
                        }
                    });
                }));
            }
        }
    }
    fn remove_window(&mut self, cx: &mut Context<Self>) {
        self.subscriptions.clear();
        self.palette = None;
        if let Some(handle) = self.window.take() {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
    }
    fn close(&mut self, invocation: u64, reason: CloseReason, cx: &mut Context<Self>) {
        if invocation != self.invocation || self.window.is_none() {
            return;
        }
        self.opening = None;
        // Restore before removing the active window, while frontmost ownership is knowable.
        if reason == CloseReason::Explicit {
            self.services.restore_frontmost(invocation);
        } else {
            self.services.discard_frontmost(invocation);
        }
        self.remove_window(cx);
    }
    pub fn shutdown(&mut self, cx: &mut Context<Self>) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        self.bridge = None;
        self.loading = None;
        self.opening = None;
        self.services.discard_frontmost(self.invocation);
        self.remove_window(cx);
        self.services.shutdown();
    }
}

pub fn palette_bounds(
    area: Bounds<Pixels>,
    rem: Pixels,
    preferred_height: Pixels,
) -> Bounds<Pixels> {
    let clearance_x = rem.min(area.size.width / 2.);
    let clearance_y = rem.min(area.size.height / 2.);
    let width = (rem * 40.)
        .min(rem * 48.)
        .min((area.size.width - clearance_x * 2.).max(Pixels::ZERO));
    let height = preferred_height.min((area.size.height - clearance_y * 2.).max(Pixels::ZERO));
    Bounds::new(
        point(
            area.left() + (area.size.width - width) / 2.,
            area.top() + (area.size.height - height) / 2.,
        ),
        size(width, height),
    )
}

struct ShellView {
    palette: Option<Entity<PaletteView>>,
    status: String,
    banner: Option<String>,
    owner: WeakEntity<AppShell>,
    invocation: u64,
    focus: FocusHandle,
}
impl ShellView {
    fn close(&self, cx: &mut App) {
        let owner = self.owner.clone();
        let invocation = self.invocation;
        cx.defer(move |cx| {
            let _ = owner.update(cx, |shell, cx| {
                shell.close(invocation, CloseReason::Explicit, cx)
            });
        });
    }
}
impl Render for ShellView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .when_some(self.banner.clone(), |this, banner| {
                this.child(
                    div()
                        .id("registration-error")
                        .role(Role::Alert)
                        .test_support()
                        .aria_label(banner.clone())
                        .p_3()
                        .child(banner),
                )
            })
            .when_some(self.palette.clone(), |this, palette| {
                this.child(div().flex_1().min_h_0().child(palette))
            })
            .when(self.palette.is_none(), |this| {
                this.child(
                    div()
                        .size_full()
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .track_focus(&self.focus)
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                            if event.keystroke.key == "escape" && !event.is_held {
                                this.close(cx);
                                cx.stop_propagation();
                            }
                        }))
                        .child(
                            div()
                                .id("startup-status")
                                .role(Role::Status)
                                .test_support()
                                .aria_label(self.status.clone())
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .child(self.status.clone()),
                        )
                        .child(
                            Button::new("close")
                                .label("Close")
                                .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
                        ),
                )
            })
    }
}
