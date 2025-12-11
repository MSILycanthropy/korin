mod container;
mod text;

use korin_runtime::{NodeContent, NodeId, Runtime};
use korin_style::Borders;

use crate::{Buffer, Rect, buffer::BufferView};

pub fn render(buffer: &mut Buffer, runtime: &Runtime) {
    let _span = tracing::debug_span!("render").entered();

    let inner = runtime.inner();

    let Some(root) = inner.root() else {
        tracing::warn!("render called with no root");
        return;
    };

    drop(inner);

    let view = buffer.view();
    render_node(buffer, &view, runtime, root);
}

fn render_node(buffer: &mut Buffer, view: &BufferView, runtime: &Runtime, id: NodeId) {
    let inner = runtime.inner();

    let Some(node) = inner.get(id) else {
        return;
    };
    let Some(rect) = inner.rect(id) else {
        return;
    };

    let style = node.computed_style;
    let content = node.content.clone();
    let children = inner.children(id);

    drop(inner);

    let rect = rect.cast::<u16>();
    let node_view = view.subview(rect);

    match &content {
        NodeContent::Container => container::render(buffer, &node_view, &style),
        NodeContent::Text(text) => text::render(buffer, &node_view, text, &style),
    }

    let inner_x = u16::from(style.borders.contains(Borders::LEFT));
    let inner_y = u16::from(style.borders.contains(Borders::TOP));
    let inner_w = node_view
        .width()
        .saturating_sub(inner_x + u16::from(style.borders.contains(Borders::RIGHT)));
    let inner_h = node_view
        .height()
        .saturating_sub(inner_y + u16::from(style.borders.contains(Borders::BOTTOM)));

    let inner_view = node_view.subview(Rect::new(inner_x, inner_y, inner_w, inner_h));

    for child in children {
        render_node(buffer, &inner_view, runtime, child);
    }
}
