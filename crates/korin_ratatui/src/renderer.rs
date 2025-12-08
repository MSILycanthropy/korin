use korin_layout::Rect;
use korin_runtime::{NodeContent, Runtime};
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

    render_node(frame, ctx, root, 0.0, 0.0);
}

pub fn render_node(
    frame: &mut Frame,
    ctx: &Runtime,
    node_id: NodeId,
    offset_x: f32,
    offset_y: f32,
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
    let abs_rect = Rect::new(abs_x, abs_y, layout_rect.width, layout_rect.height);

    let rat_rect = to_rat_rect(abs_rect);
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

    for child_id in children {
        render_node(frame, ctx, child_id, abs_x, abs_y);
    }
}
