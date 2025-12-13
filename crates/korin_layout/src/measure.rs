use crate::engine::NodeMeasure;

use taffy::{AvailableSpace, NodeId, Size, Style};
use unicode_width::UnicodeWidthStr;

pub fn taffy_measure(
    known_size: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    _node_id: NodeId,
    ctx: Option<&mut NodeMeasure>,
    _style: &Style<String>,
) -> Size<f32> {
    let Some(ctx) = ctx else {
        return taffy::Size::ZERO;
    };

    let Some(text) = &ctx.0 else {
        return taffy::Size::ZERO;
    };

    if let Some(width) = known_size.width {
        return measure_text(text, width);
    }

    let available_width = match available_space.width {
        AvailableSpace::Definite(w) => w,
        AvailableSpace::MinContent => 1.0,
        AvailableSpace::MaxContent => f32::MAX,
    };

    measure_text(text, available_width)
}

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn measure_text(text: &str, available_width: f32) -> Size<f32> {
    let max_width = available_width.max(1.0);
    let mut total_height = 0.0;
    let mut max_line_width: f32 = 0.0;

    for line in text.lines() {
        let line_width = line.width() as f32;

        if line_width == 0.0 {
            total_height += 1.0;
            continue;
        }

        if line_width <= max_width {
            total_height += 1.0;
            max_line_width = max_line_width.max(line_width);
        } else {
            let wrapped_lines = (line_width + max_width - 1.0) / max_width;
            total_height += wrapped_lines;
            max_line_width = max_width;
        }
    }

    dbg!(max_line_width);

    Size {
        width: max_line_width,
        height: total_height.max(1.0),
    }
}
