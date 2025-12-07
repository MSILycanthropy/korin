use korin_layout::{Point, Rect};

#[derive(Clone, Debug, Default)]
pub struct RenderState {
    pub offset: Point,
    pub clip: Rect,
}

impl RenderState {
    pub const fn new(clip: Rect) -> Self {
        Self {
            offset: Point { x: 0.0, y: 0.0 },
            clip,
        }
    }

    pub fn translate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + self.offset.x,
            rect.y + self.offset.y,
            rect.width,
            rect.height,
        )
    }

    pub fn apply_clip(&self, rect: Rect) -> Option<Rect> {
        let x1 = rect.x.max(self.clip.x);
        let y1 = rect.y.max(self.clip.y);
        let x2 = (rect.x + rect.width).min(self.clip.x + self.clip.width);
        let y2 = (rect.y + rect.height).min(self.clip.y + self.clip.height);

        if x2 > x1 && y2 > y1 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    pub fn transform(&self, rect: Rect) -> Option<Rect> {
        let translated = self.translate(rect);
        self.apply_clip(translated)
    }
}
