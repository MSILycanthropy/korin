mod container;
mod text;

use korin_runtime::{Node, NodeContent};

use crate::{Buffer, buffer::BufferView};

pub fn render_node(buffer: &mut Buffer, view: &BufferView, node: &Node) {
    let style = &node.computed_style;

    match &node.content {
        NodeContent::Container => container::render(buffer, view, node),
        NodeContent::Text(text) => text::render(buffer, view, text, style),
    }
}
