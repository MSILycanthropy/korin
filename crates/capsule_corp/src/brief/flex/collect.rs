use crate::{
    AlignSelf, AvailableSpace, CapsuleDocument, CapsuleNode, ComputedStyle, Constraints, Dimension,
    Display, Edges, FlexDirection,
    brief::{engine::compute_node_box, flex::core::FlexItem},
};

pub fn collect_flex_items<D: CapsuleDocument>(
    document: &mut D,
    container_id: D::NodeId,
    direction: FlexDirection,
    available_main: AvailableSpace,
    available_cross: AvailableSpace,
) -> Vec<FlexItem<D::NodeId>> {
    let is_row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
    let available_main_px = available_main.as_definite().unwrap_or(0);

    let children: Vec<_> = document.children(container_id).collect();
    let mut items = Vec::with_capacity(children.len());

    for child in children {
        if document.get_node(child).text_content().is_some() {
            let child_constraints = Constraints::new(available_main, available_cross);
            let resolved_box = compute_node_box(document, child, child_constraints, true);

            let (main_size, cross_size) = if is_row {
                (
                    resolved_box.border_box_size().width,
                    resolved_box.border_box_size().height,
                )
            } else {
                (
                    resolved_box.border_box_size().height,
                    resolved_box.border_box_size().width,
                )
            };

            items.push(FlexItem {
                node_id: child,
                align_self: AlignSelf::Auto,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                flex_basis: main_size,
                min_main_size: main_size,
                max_main_size: Some(main_size),
                hypothetical_main_size: main_size,
                margin: Edges::ZERO,
                resolved_box,
                frozen: true, // text doesn't grow/shrink
                main_size,
                cross_size,
                main_position: 0,
                cross_position: 0,
            });

            continue;
        }

        let style = document
            .get_node(child)
            .computed_style()
            .cloned()
            .expect("non-text node must have style");

        if matches!(style.display, Display::None) {
            continue;
        }

        let margin = style.margin.resolve(available_main_px);
        let flex_basis = resolve_flex_basis(&style.flex_basis, is_row, &style, available_main_px);

        let (min_main, max_main) = if is_row {
            (
                style.min_width.resolve(available_main_px).unwrap_or(0),
                style.max_width.resolve(available_main_px),
            )
        } else {
            (
                style.min_height.resolve(available_main_px).unwrap_or(0),
                style.max_height.resolve(available_main_px),
            )
        };

        let hypothetical_main_size = clamp(flex_basis, min_main, max_main);

        let child_constraints = if is_row {
            Constraints::new(
                AvailableSpace::Definite(hypothetical_main_size),
                available_cross,
            )
        } else {
            Constraints::new(
                available_cross,
                AvailableSpace::Definite(hypothetical_main_size),
            )
        };

        let resolved_box = compute_node_box(document, child, child_constraints, true);

        let cross_size = if is_row {
            resolved_box.border_box_size().height
        } else {
            resolved_box.border_box_size().width
        };

        items.push(FlexItem {
            node_id: child,
            align_self: style.align_self,
            flex_grow: style.flex_grow,
            flex_shrink: style.flex_shrink,
            flex_basis,
            min_main_size: min_main,
            max_main_size: max_main,
            hypothetical_main_size,
            margin,
            resolved_box,
            frozen: false,
            main_size: hypothetical_main_size,
            cross_size,
            main_position: 0,
            cross_position: 0,
        });
    }

    items
}

fn resolve_flex_basis(
    flex_basis: &Dimension,
    is_row: bool,
    style: &ComputedStyle,
    available: u16,
) -> u16 {
    match flex_basis {
        Dimension::Length(length) => length.resolve(available),
        Dimension::Auto => {
            let size = if is_row { &style.width } else { &style.height };

            match size {
                Dimension::Length(length) => length.resolve(available),
                _ => 0,
            }
        }
        Dimension::None => 0,
    }
}

fn clamp(value: u16, min: u16, max: Option<u16>) -> u16 {
    let value = value.max(min);
    max.map_or(value, |v| value.min(v))
}
