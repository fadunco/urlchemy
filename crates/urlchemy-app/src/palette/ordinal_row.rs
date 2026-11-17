use super::ordinals::{OrdinalFrame, OrdinalGeometry, OrdinalMapping};
use gpui_kit::{
    base::VirtualListScrollHandle,
    component::{ActiveTheme, Selectable, label::Label, list::ListItem},
    *,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use urlchemy_core::EntryId;

pub(super) struct OrdinalPresentation {
    pub(super) scroll: VirtualListScrollHandle,
    pub(super) revision: u64,
    pub(super) displayed: Option<OrdinalMapping>,
    pub(super) row_height: Option<Pixels>,
    frame: Option<OrdinalFrame>,
    collecting: bool,
}
impl Default for OrdinalPresentation {
    fn default() -> Self {
        Self {
            scroll: VirtualListScrollHandle::new(),
            revision: 0,
            displayed: None,
            row_height: None,
            frame: None,
            collecting: false,
        }
    }
}
impl OrdinalPresentation {
    pub(super) fn geometry(&self, window: &Window) -> OrdinalGeometry {
        OrdinalGeometry::new(
            self.revision,
            self.scroll.bounds(),
            self.scroll.offset(),
            window.viewport_size(),
            window.rem_size(),
        )
    }
    fn record(
        &mut self,
        ix: usize,
        id: EntryId,
        bounds: Bounds<Pixels>,
        window: &Window,
    ) -> Option<u8> {
        if !self.collecting {
            return None;
        }
        let geometry = self.geometry(window);
        self.frame
            .get_or_insert_with(|| OrdinalFrame::new(geometry))
            .record(ix, id, bounds, window.content_mask().bounds)
    }
}
type Presentation = Rc<RefCell<OrdinalPresentation>>;

pub(super) struct OrdinalRow {
    item: Option<ListItem>,
    sample: Option<(usize, EntryId)>,
    presentation: Presentation,
    number: Rc<Cell<Option<u8>>>,
}
impl OrdinalRow {
    pub(super) fn new(
        item: ListItem,
        sample: Option<(usize, EntryId)>,
        presentation: Presentation,
    ) -> Self {
        Self {
            item: Some(item),
            sample,
            presentation,
            number: Rc::new(Cell::new(None)),
        }
    }
}
impl Selectable for OrdinalRow {
    fn is_selected(&self) -> bool {
        self.item.as_ref().is_some_and(Selectable::is_selected)
    }
    fn selected(mut self, selected: bool) -> Self {
        self.item = self.item.take().map(|item| item.selected(selected));
        self
    }
    fn secondary_selected(mut self, selected: bool) -> Self {
        self.item = self
            .item
            .take()
            .map(|item| item.secondary_selected(selected));
        self
    }
}
impl Element for OrdinalRow {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let number = self.number.clone();
        let item = self.item.take().expect("row layout requested once");
        let item = if self.sample.is_some() {
            item.suffix(move |_, _| OrdinalHint {
                number: number.clone(),
            })
        } else {
            item
        };
        let mut inner = div()
            .id("ordinal-row")
            .test_support()
            .w_full()
            .child(item)
            .into_any_element();
        (inner.request_layout(window, cx), inner)
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        inner: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        // The complete row frame is available before its hint prepaints.
        self.presentation.borrow_mut().row_height = Some(bounds.size.height);
        self.number.set(self.sample.and_then(|(ix, id)| {
            self.presentation
                .borrow_mut()
                .record(ix, id, bounds, window)
        }));
        inner.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        inner: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        inner.paint(window, cx);
    }
}

struct OrdinalHint {
    number: Rc<Cell<Option<u8>>>,
}
impl Element for OrdinalHint {
    type RequestLayoutState = AnyElement;
    type PrepaintState = AnyElement;
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let mut slot = div().w_8().h_5().flex_shrink_0().into_any_element();
        (slot.request_layout(window, cx), slot)
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        // The fixed slot keeps layout independent from viewport numbering.
        let mut slot = div()
            .id("ordinal-slot")
            .test_support()
            .size_full()
            .text_sm()
            .text_color(cx.theme().muted_foreground);
        if let Some(number) = self.number.get() {
            let text = format!("⌘{number}");
            slot = slot.child(
                div()
                    .id("ordinal-hint")
                    .test_support()
                    .aria_label(text.clone())
                    .child(Label::new(text)),
            );
        }
        let mut slot = slot.into_any_element();
        slot.layout_as_root(
            size(
                AvailableSpace::Definite(bounds.size.width),
                AvailableSpace::Definite(bounds.size.height),
            ),
            window,
            cx,
        );
        slot.prepaint_at(bounds.origin, window, cx);
        slot
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut AnyElement,
        slot: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        slot.paint(window, cx);
    }
}

pub(super) struct OrdinalList {
    inner: Option<AnyElement>,
    presentation: Presentation,
}
impl IntoElement for OrdinalRow {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl IntoElement for OrdinalHint {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl IntoElement for OrdinalList {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl OrdinalList {
    pub(super) fn new(inner: impl IntoElement, presentation: Presentation) -> Self {
        Self {
            inner: Some(inner.into_any_element()),
            presentation,
        }
    }
}
impl Element for OrdinalList {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, AnyElement) {
        let mut inner = self.inner.take().expect("list layout requested once");
        (inner.request_layout(window, cx), inner)
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        inner: &mut AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Scroll-only updates can prepaint rows without rerendering the palette.
        {
            let mut p = self.presentation.borrow_mut();
            p.frame = None;
            p.displayed = None;
            p.collecting = true;
        }
        inner.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        inner: &mut AnyElement,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        inner.paint(window, cx);
        // Publish once, after every hint in this presentation has painted.
        let mut p = self.presentation.borrow_mut();
        p.collecting = false;
        p.displayed = p.frame.take().and_then(OrdinalFrame::finish);
    }
}
