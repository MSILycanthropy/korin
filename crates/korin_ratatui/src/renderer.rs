use korin_layout::Rect;
use korin_runtime::{NodeContent, Runtime, RuntimeContext};
use korin_tree::NodeId;
use ratatui::{
    Frame,
    text::Text,
    widgets::{Block, Paragraph},
};

use crate::{
    converstions::{
        to_rat_border_type, to_rat_borders, to_rat_color, to_rat_rect, to_rat_style_text,
    },
    state::RenderState,
};

pub fn render(frame: &mut Frame, ctx: &Runtime) {
    let size = frame.area();
    let clip = Rect::new(0.0, 0.0, f32::from(size.width), f32::from(size.height));

    let state = RenderState::new(clip);

    let runtime = ctx.inner();

    let Some(root) = runtime.root() else {
        return;
    };

    drop(runtime);

    render_node(frame, ctx, root, &state);
}

pub fn render_node(frame: &mut Frame, ctx: &Runtime, node_id: NodeId, state: &RenderState) {
    let runtime = ctx.inner();

    let Some(node) = runtime.get(node_id) else {
        return;
    };

    let Some(layout_rect) = runtime.rect(node_id) else {
        return;
    };

    let Some(rect) = state.transform(layout_rect) else {
        return;
    };

    let rat_rect = to_rat_rect(rect);
    let style = node.style;
    let content = node.content.clone();

    drop(runtime);

    match &content {
        NodeContent::Container => {
            let block = Block::default()
                .borders(to_rat_borders(style.borders))
                .border_type(to_rat_border_type(style.border_style))
                .border_style(to_rat_color(style.border_color));

            frame.render_widget(block, rat_rect);
        }
        NodeContent::Text(text) => {
            let text = Text::raw(text).style(to_rat_style_text(&style));
            let paragraph = Paragraph::new(text);

            frame.render_widget(paragraph, rat_rect);
        }
    }
}
