use crate::{
    AvailableSpace, CapsuleDocument, CapsuleNode, Constraints, Display, Edges, Layout, Point, Size,
    brief::{box_model::ResolvedBox, flex, resolve::resolve_size_constraints, text::measure_text},
};

pub fn compute_layout<D: CapsuleDocument>(document: &mut D, root: D::NodeId, viewport: Size) {
    let constraints = Constraints::from_size(viewport);

    let resolved_box = compute_node_box(document, root, constraints, true);

    document.get_node_mut(root).set_layout(Layout {
        order: 0,
        location: Point::ZERO,
        scrollbar_size: Size::ZERO,
        resolved_box,
    });
}

pub fn compute_node_box<D: CapsuleDocument>(
    document: &mut D,
    node: D::NodeId,
    constraints: Constraints,
    force: bool,
) -> ResolvedBox {
    let node_id = node;
    let node = document.get_node(node);

    if !force && !node.needs_layout() {
        return node.layout().resolved_box;
    }

    if let Some(text) = node.text_content() {
        let parent_style = document
            .parent(node_id)
            .and_then(|parent| document.get_node(parent).computed_style())
            .cloned()
            .unwrap_or_default();
        let size = measure_text(text, parent_style.white_space, constraints.width);
        document.get_node_mut(node_id).clear_needs_layout();
        return size.into();
    }

    let node = document.get_node_mut(node_id);
    node.clear_needs_layout();

    let style = node
        .computed_style()
        .cloned()
        .expect("element node must have computed style");

    if matches!(style.display, Display::None) {
        node.set_layout(Layout::ZERO);
        return ResolvedBox::ZERO;
    }

    let parent_width = constraints.width.as_definite().unwrap_or(0);
    let parent_height = constraints.height.as_definite();

    let size_constraints = resolve_size_constraints(&style, parent_width, parent_height);

    let margin = style.margin.resolve(parent_width);
    let border = style.border_style.to_widths();
    let padding = style.padding.resolve(parent_width);

    let border_padding_h = border.horizontal().saturating_add(padding.horizontal());
    let border_padding_v = border.vertical().saturating_add(padding.vertical());
    let content_constraints = constraints.shrink(border_padding_h, border_padding_v);

    let content_size = match style.display {
        Display::Block => layout_block(document, node_id, content_constraints),
        Display::Flex => flex::layout(document, node_id, &style, content_constraints),
        Display::Inline => layout_inline(document, node_id, content_constraints),
        Display::Grid => layout_grid(document, node_id, content_constraints),
        Display::None => unreachable!(),
    };

    let final_content_size = Size::new(
        size_constraints.width.map_or_else(
            || {
                size_constraints
                    .clamp_width(content_size.width.saturating_add(border_padding_h))
                    .saturating_sub(border_padding_h)
            },
            |w| size_constraints.clamp_width(w),
        ),
        size_constraints.height.map_or_else(
            || {
                size_constraints
                    .clamp_height(content_size.height.saturating_add(border_padding_v))
                    .saturating_sub(border_padding_v)
            },
            |h| size_constraints.clamp_height(h),
        ),
    );

    ResolvedBox {
        margin,
        border,
        padding,
        content_size: final_content_size,
    }
}

fn layout_block<D: CapsuleDocument>(
    document: &mut D,
    node: D::NodeId,
    constraints: Constraints,
) -> Size {
    let available_width = constraints.width.as_definite().unwrap_or(0);
    let mut y = 0u16;

    let children: Vec<_> = document.children(node).collect();

    for child in children {
        let style = document.get_node(child).computed_style();

        if style.is_some_and(|s| matches!(s.display, Display::None)) {
            continue;
        }

        let child_margin = style.map_or(Edges::ZERO, |s| s.margin.resolve(available_width));

        y = y.saturating_add(child_margin.top);

        let child_available_width = available_width
            .saturating_sub(child_margin.left)
            .saturating_sub(child_margin.right);

        let child_constraints = Constraints::new(
            AvailableSpace::Definite(child_available_width),
            constraints.height.shrink(y),
        );

        let child_box = compute_node_box(document, child, child_constraints, false);

        document.get_node_mut(child).set_layout(Layout {
            order: 0,
            location: Point::new(child_margin.left, y),
            scrollbar_size: Size::ZERO,
            resolved_box: ResolvedBox {
                margin: child_margin,
                ..child_box
            },
        });

        y = y.saturating_add(child_box.border_box_size().height);
        y = y.saturating_add(child_margin.bottom);
    }

    Size::new(available_width, y)
}

fn layout_inline<D: CapsuleDocument>(
    document: &mut D,
    node: D::NodeId,
    constraints: Constraints,
) -> Size {
    let available_width = constraints.width.as_definite().unwrap_or(u16::MAX);

    let mut x = 0u16;
    let mut y = 0u16;
    let mut line_height = 0u16;
    let mut max_width = 0u16;

    let children: Vec<_> = document.children(node).collect();

    for child in children {
        let style = document.get_node(child).computed_style();

        if style.is_some_and(|s| matches!(s.display, Display::None)) {
            continue;
        }

        let child_margin = style.map_or(Edges::ZERO, |s| s.margin.resolve(available_width));
        let child_constraints = Constraints::new(
            AvailableSpace::Definite(
                available_width
                    .saturating_sub(child_margin.left)
                    .saturating_sub(child_margin.right),
            ),
            constraints.height,
        );
        let child_box = compute_node_box(document, child, child_constraints, false);

        let border_box_size = child_box.border_box_size();
        let child_width = border_box_size
            .width
            .saturating_add(child_margin.left)
            .saturating_add(child_margin.right);
        let child_height = border_box_size
            .height
            .saturating_add(child_margin.top)
            .saturating_add(child_margin.bottom);

        if x > 0 && x + child_width > available_width {
            y = y.saturating_add(line_height);
            x = 0;
            line_height = 0;
        }

        let child_x = x.saturating_add(child_margin.left);
        let child_y = y.saturating_add(child_margin.top);

        document.get_node_mut(child).set_layout(Layout {
            order: 0,
            location: Point::new(child_x, child_y),
            scrollbar_size: Size::ZERO,
            resolved_box: ResolvedBox {
                margin: child_margin,
                ..child_box
            },
        });

        x = x.saturating_add(child_width);
        max_width = max_width.max(x);
        line_height = line_height.max(child_height);
    }

    y = y.saturating_add(line_height);

    Size::new(max_width, y)
}

fn layout_grid<D: CapsuleDocument>(
    _document: &mut D,
    _node: D::NodeId,
    _constraints: Constraints,
) -> Size {
    todo!("grid")
}
