use std::io;

use capsule_corp::CapsuleDocument;
use capsule_corp::ComputedStyle;
use capsule_corp::CustomPropertiesMap;
use capsule_corp::Display;
use capsule_corp::Size;
use ginyu_force::pose;
use korin::{Document, Mountable, View, div, text, view::BuildContext};

fn main() -> io::Result<()> {
    let mut document = Document::new();
    let root = document.root();

    let div = div(text("Hello World!")).attribute(
        pose!("style"),
        "background: blue; padding: 1; text-align: center; display: flex; width: 100%; justify-content: center;",
    );

    // TODO: We.. probably shouldn't have to do this? This shouldn't really be relevant for paint should it?
    document.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
        CustomPropertiesMap::default(),
    );

    let mut build_context = BuildContext::new(&mut document);
    let mut state = div.build(&mut build_context);
    state.mount(root, None, &mut document);

    capsule_corp::compute_styles(&mut document);
    capsule_corp::compute_layout(&mut document, root, Size::new(111, 13));

    korin::run_once(&document)
}
