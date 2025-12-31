//! Convenience functions for creating elements.
//!
//! Instead of:
//! ```ignore
//! ElementView::new(pose!("div"), children)
//! ```
//!
//! You can write:
//! ```ignore
//! div(children)
//! ```

use ginyu_force::pose;

use crate::view::{ElementView, View};

macro_rules! define_elements {
    ($($name:ident),* $(,)?) => {
        $(
            #[doc = concat!("Create a `<", stringify!($name), ">` element.")]
            pub fn $name<C: View>(children: C) -> ElementView<C> {
                ElementView::new(pose!(stringify!($name)), children)
            }
        )*
    };
}

// Layout elements
define_elements! {
    div,
    span,
    section,
    header,
    footer,
    nav,
    main,
    aside,
    article,
}

// Text elements
define_elements! {
    p,
    h1,
    h2,
    h3,
    h4,
    h5,
    h6,
    code,
    em,
    strong,
    small,
    mark,
    del,
    ins,
    sub,
    sup,
}

// Form elements
define_elements! {
    button,
    input,
    select,
    option,
    optgroup,
    textarea,
    label,
    form,
    fieldset,
    legend,
}

// List elements
define_elements! {
    ul,
    ol,
    li,
    dl,
    dt,
    dd,
    menu,
}

// Table elements
define_elements! {
    table,
    thead,
    tbody,
    tfoot,
    tr,
    th,
    td,
    caption,
    colgroup,
    col,
}

// Interactive elements
define_elements! {
    details,
    summary,
    dialog,
}

// Other elements
define_elements! {
    slot,
    template,
    progress,
    meter,
}

/// Create a text node (convenience wrapper around `TextView`)
pub fn text(content: impl Into<String>) -> crate::view::TextView {
    crate::view::TextView::new(content)
}
