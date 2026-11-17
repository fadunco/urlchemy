use gpui_kit::Pixels;

pub(super) fn preferred_height(
    header: Pixels,
    question: Option<Pixels>,
    body: Pixels,
    footer: Pixels,
    rem: Pixels,
) -> Pixels {
    let gaps = if question.is_some() { 3. } else { 2. };
    header
        + question.unwrap_or(Pixels::ZERO)
        + body.min(rem * 24.)
        + footer
        + rem * (2. + gaps * 0.75)
}

#[derive(Default)]
pub(super) struct RegionHeights {
    pub(super) header: Option<Pixels>,
    pub(super) question: Option<Pixels>,
    pub(super) body: Option<Pixels>,
    pub(super) footer: Option<Pixels>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::px;

    #[test]
    fn named_regions_determine_preferred_height() {
        assert_eq!(
            preferred_height(px(30.), Some(px(60.)), px(400.), px(30.), px(10.)),
            px(402.5)
        );
        assert_eq!(
            preferred_height(px(30.), None, px(400.), px(30.), px(10.)),
            px(335.)
        );
        assert_eq!(
            preferred_height(px(30.), Some(px(60.)), px(400.), px(30.), px(20.)),
            px(605.)
        );
    }
}
