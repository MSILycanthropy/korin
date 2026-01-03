use crate::{Document, NodeData};
use indextree::NodeId;
use std::fmt::Write as _;

impl Document {
    #[must_use]
    pub fn debug_html(&self) -> String {
        let mut output = String::new();
        self.debug_node(self.root, &mut output, 0);
        output
    }

    fn debug_node(&self, id: NodeId, output: &mut String, depth: usize) {
        let Some(node) = self.get(id) else {
            return;
        };

        let indent = "  ".repeat(depth);

        match &node.data {
            NodeData::Root => {
                for child in self.children(id) {
                    self.debug_node(child, output, depth);
                }
            }
            NodeData::Element(element) => {
                output.push_str(&indent);
                output.push('<');
                output.push_str(element.tag.as_str());

                if let Some(id) = element.id {
                    let _ = write!(output, r#" id="{}""#, id.as_str());
                }

                if !element.classes.is_empty() {
                    let classes: Vec<_> = element.classes.iter().map(|c| c.as_str()).collect();

                    let _ = write!(output, r#" class="{}""#, classes.join(" "));
                }

                for (key, value) in &element.attributes {
                    let _ = write!(output, r#" {}="{}""#, key.as_str(), value);
                }

                output.push_str(">\n");

                for child in self.children(id) {
                    self.debug_node(child, output, depth + 1);
                }

                output.push_str(&indent);
                output.push_str("</");
                output.push_str(element.tag.as_str());
                output.push_str(">\n");
            }
            NodeData::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    output.push_str(&indent);
                    output.push_str(trimmed);
                    output.push('\n');
                }
            }
            NodeData::Marker => output.push_str("<-- MARKER -->"),
        }
    }
}
