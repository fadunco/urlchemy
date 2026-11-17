use gpui_kit::{Bounds, Pixels, Point, Size};
use std::collections::BTreeSet;
use urlchemy_core::EntryId;

#[derive(Clone, PartialEq)]
pub(super) struct OrdinalGeometry {
    revision: u64,
    viewport: Bounds<Pixels>,
    offset: Point<Pixels>,
    window_size: Size<Pixels>,
    rem: Pixels,
}
impl OrdinalGeometry {
    pub(super) fn new(
        revision: u64,
        viewport: Bounds<Pixels>,
        offset: Point<Pixels>,
        window_size: Size<Pixels>,
        rem: Pixels,
    ) -> Self {
        Self {
            revision,
            viewport,
            offset,
            window_size,
            rem,
        }
    }
}
pub(super) struct OrdinalFrame {
    geometry: OrdinalGeometry,
    entries: Vec<EntryId>,
    seen: BTreeSet<EntryId>,
    last_ix: Option<usize>,
    coherent: bool,
}
impl OrdinalFrame {
    pub(super) fn new(geometry: OrdinalGeometry) -> Self {
        Self {
            geometry,
            entries: Vec::new(),
            seen: BTreeSet::new(),
            last_ix: None,
            coherent: true,
        }
    }
    pub(super) fn record(
        &mut self,
        result_ix: usize,
        id: EntryId,
        bounds: Bounds<Pixels>,
        clip: Bounds<Pixels>,
    ) -> Option<u8> {
        if self.last_ix.is_some_and(|last| result_ix <= last) || !self.seen.insert(id) {
            self.coherent = false;
        }
        self.last_ix = Some(result_ix);
        let visible = self.geometry.viewport.intersect(&clip);
        if !self.coherent
            || self.entries.len() == 9
            || bounds.size.width <= Pixels::ZERO
            || bounds.size.height <= Pixels::ZERO
            || visible.size.width <= Pixels::ZERO
            || visible.size.height <= Pixels::ZERO
            || bounds.intersect(&visible) != bounds
        {
            return None;
        }
        self.entries.push(id);
        Some(self.entries.len() as u8)
    }
    pub(super) fn finish(self) -> Option<OrdinalMapping> {
        self.coherent.then_some(OrdinalMapping {
            geometry: self.geometry,
            entries: self.entries,
        })
    }
}
pub(super) struct OrdinalMapping {
    geometry: OrdinalGeometry,
    entries: Vec<EntryId>,
}
impl OrdinalMapping {
    pub(super) fn resolve(&self, number: u8, current: &OrdinalGeometry) -> Option<EntryId> {
        if *current != self.geometry {
            return None;
        }
        self.entries.get(number.checked_sub(1)? as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{OrdinalFrame, OrdinalGeometry};
    use gpui_kit::{Bounds, Pixels, point, px, size};
    use std::path::Path;
    use urlchemy_core::{EntryId, config::load_bytes};

    fn ids() -> Vec<EntryId> {
        let mut source = "navigation:\n".to_owned();
        for n in 0..12 {
            source.push_str(&format!(
                "  - folder: {{label: Folder {n}, children: []}}\n"
            ));
        }
        load_bytes(Path::new("fixture"), source.as_bytes())
            .unwrap()
            .roots()
            .iter()
            .map(|e| e.id())
            .collect()
    }
    fn rect(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }
    fn geometry() -> OrdinalGeometry {
        OrdinalGeometry::new(
            1,
            rect(0., 10., 100., 100.),
            point(px(0.), px(0.)),
            size(px(200.), px(200.)),
            px(14.),
        )
    }
    #[test]
    fn fully_visible_rows_receive_at_most_nine_ordinals() {
        let ids = ids();
        let mut frame = OrdinalFrame::new(geometry());
        for (ix, id) in ids.iter().copied().enumerate() {
            let number = frame.record(
                ix,
                id,
                rect(0., 5. + ix as f32 * 10., 100., 10.),
                rect(0., 10., 100., 100.),
            );
            assert_eq!(
                number,
                if (1..=9).contains(&ix) {
                    Some(ix as u8)
                } else {
                    None
                }
            );
        }
        let map = frame.finish().unwrap();
        assert_eq!(map.resolve(1, &geometry()), Some(ids[1]));
        assert_eq!(map.resolve(9, &geometry()), Some(ids[9]));
        assert_eq!(map.resolve(0, &geometry()), None);
        assert_eq!(map.resolve(10, &geometry()), None);
    }
    #[test]
    fn containment_uses_full_row_and_effective_clip() {
        let id = ids()[0];
        for (bounds, clip, expected) in [
            (rect(0., 10., 100., 10.), rect(0., 10., 100., 100.), Some(1)),
            (rect(0., 9., 100., 10.), rect(0., 10., 100., 100.), None),
            (rect(-1., 10., 100., 10.), rect(0., 10., 100., 100.), None),
            (rect(0., 10., 0., 10.), rect(0., 10., 100., 100.), None),
            (rect(0., 10., 100., 10.), rect(0., 10., 100., 0.), None),
            (rect(0., 10., 100., 10.), rect(0., 11., 100., 100.), None),
            (
                rect(0., 100., 100., 10.),
                rect(0., 10., 100., 100.),
                Some(1),
            ),
        ] {
            let mut frame = OrdinalFrame::new(geometry());
            assert_eq!(frame.record(0, id, bounds, clip), expected);
        }
    }
    #[test]
    fn duplicate_or_reordered_samples_reject_the_frame() {
        let ids = ids();
        for samples in [
            vec![(0, ids[0]), (1, ids[0])],
            vec![(0, ids[0]), (0, ids[1])],
            vec![(1, ids[0]), (0, ids[1])],
        ] {
            let mut frame = OrdinalFrame::new(geometry());
            for (ix, id) in samples {
                frame.record(ix, id, rect(0., 10., 100., 10.), rect(0., 10., 100., 100.));
            }
            assert!(frame.finish().is_none());
        }
    }
    #[test]
    fn mapping_rejects_changed_geometry() {
        let mut frame = OrdinalFrame::new(geometry());
        frame.record(
            0,
            ids()[0],
            rect(0., 10., 100., 10.),
            rect(0., 10., 100., 100.),
        );
        let map = frame.finish().unwrap();
        let g = geometry();
        for changed in [
            OrdinalGeometry::new(2, g.viewport, g.offset, g.window_size, g.rem),
            OrdinalGeometry::new(1, rect(0., 11., 100., 100.), g.offset, g.window_size, g.rem),
            OrdinalGeometry::new(1, g.viewport, point(px(0.), px(-1.)), g.window_size, g.rem),
            OrdinalGeometry::new(1, g.viewport, g.offset, size(px(201.), px(200.)), g.rem),
            OrdinalGeometry::new(1, g.viewport, g.offset, g.window_size, px(20.)),
        ] {
            assert_eq!(map.resolve(1, &changed), None);
        }
    }
}
