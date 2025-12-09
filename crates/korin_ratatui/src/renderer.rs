use korin_layout::Rect;
use korin_runtime::{NodeContent, Runtime};
use korin_style::Borders;
use korin_tree::NodeId;
use ratatui::{
    Frame,
    text::Text,
    widgets::{Block, Paragraph},
};

use crate::converstions::{
    to_rat_border_type, to_rat_borders, to_rat_color, to_rat_rect, to_rat_style_text,
};

pub fn render(frame: &mut Frame, ctx: &Runtime) {
    let runtime = ctx.inner();

    let Some(root) = runtime.root() else {
        return;
    };

    drop(runtime);

    let frame_size = frame.area();
    let clip = Rect::new(
        f32::from(frame_size.x),
        f32::from(frame_size.y),
        f32::from(frame_size.width),
        f32::from(frame_size.height),
    );

    render_node(frame, ctx, root, 0.0, 0.0, clip);
}

pub fn render_node(
    frame: &mut Frame,
    ctx: &Runtime,
    node_id: NodeId,
    offset_x: f32,
    offset_y: f32,
    clip: Rect,
) {
    let runtime = ctx.inner();

    let Some(node) = runtime.get(node_id) else {
        return;
    };

    let Some(layout_rect) = runtime.rect(node_id) else {
        return;
    };

    let abs_x = offset_x + layout_rect.x;
    let abs_y = offset_y + layout_rect.y;

    let clipped = clip_rect(
        Rect::new(abs_x, abs_y, layout_rect.width, layout_rect.height),
        clip,
    );

    if clipped.width <= 0.0 || clipped.height <= 0.0 {
        return;
    }

    let rat_rect = to_rat_rect(clipped);
    let style = node.computed_style;
    let content = node.content.clone();
    let children = runtime.children(node_id);

    drop(runtime);

    match &content {
        NodeContent::Container => {
            let block = Block::default()
                .borders(to_rat_borders(style.borders))
                .border_type(to_rat_border_type(style.border_style))
                .border_style(to_rat_color(style.border_color))
                .style(ratatui::style::Style::default().bg(to_rat_color(style.background)));

            frame.render_widget(block, rat_rect);
        }
        NodeContent::Text(text) => {
            let text = Text::raw(text).style(to_rat_style_text(&style));
            let paragraph = Paragraph::new(text);

            frame.render_widget(paragraph, rat_rect);
        }
    }

    let inner_offset_x = if style.borders.contains(Borders::LEFT) {
        1.0
    } else {
        0.0
    };
    let inner_offset_y = if style.borders.contains(Borders::TOP) {
        1.0
    } else {
        0.0
    };

    for child_id in children {
        render_node(
            frame,
            ctx,
            child_id,
            abs_x + inner_offset_x,
            abs_y + inner_offset_y,
            clip,
        );
    }
}

fn clip_rect(rect: Rect, clip: Rect) -> Rect {
    let x1 = rect.x.max(clip.x);
    let y1 = rect.y.max(clip.y);
    let x2 = (rect.x + rect.width).min(clip.x + clip.width);
    let y2 = (rect.y + rect.height).min(clip.y + clip.height);

    Rect::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}
