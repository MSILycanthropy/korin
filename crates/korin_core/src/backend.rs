use dom_events::Event;
use markup5ever::LocalName;

use crate::{Attributes, Size};

pub trait TextMeasure {
    fn measure(&self, text: &str) -> Size<f32>;
}

pub trait ElementData: Default {
    fn handle_event<T>(&mut self, _event: &Event<T>) -> bool {
        false
    }
}

pub trait Backend {
    type TextMeasure: TextMeasure;
    type ElementData: ElementData;

    fn text_measure(&self) -> &Self::TextMeasure;

    fn resolve_element(&mut self, tag: &LocalName, attrs: &Attributes) -> Self::ElementData;
}
