use std::fmt;

use korin_layout::{Point, Size};
use korin_style::{PseudoState, Style};

#[derive(Clone)]
pub enum NodeContent {
    Container,
    Text(String),
}

impl fmt::Debug for NodeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Container => write!(f, "Container"),
            Self::Text(s) if s.len() <= 20 => write!(f, "Text({s:?})"),
            Self::Text(s) => write!(f, "Text({:?}...)", &s[..20]),
        }
    }
}

impl fmt::Display for NodeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Container => write!(f, "container"),
            Self::Text(s) if s.len() <= 20 => write!(f, "text: {s}"),
            Self::Text(s) => {
                let truncated: String = s.chars().take(20).collect();
                write!(f, "text: {truncated}...")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub content: NodeContent,
    pub style: Style,
    pub computed_style: Style,
    pub scroll_offset: Point,
    pub content_size: Size,
    pub pseudo_state: PseudoState,
}

impl Node {
    #[must_use]
    pub fn container() -> Self {
        Self::default()
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: NodeContent::Text(text.into()),
            ..Default::default()
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            content: NodeContent::Container,
            style: Style::default(),
            computed_style: Style::default(),
            scroll_offset: Point::default(),
            content_size: Size::default(),
            pseudo_state: PseudoState::NONE,
        }
    }
}
