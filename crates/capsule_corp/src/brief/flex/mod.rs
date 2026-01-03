use crate::{
    CapsuleDocument, CapsuleNode, ComputedStyle, Constraints, FlexDirection, Layout, Point, Size,
    brief::{box_model::ResolvedBox, flex::core::FlexItem},
};

mod align;
mod collect;
mod core;
mod cross;
mod flexible;
mod lines;

/// Perform flex layout on a container
///
/// Returns the content size
pub fn layout<D: CapsuleDocument>(
    document: &mut D,
    node_id: D::NodeId,
    style: &ComputedStyle,
    constraints: Constraints,
) -> Size {
    use FlexDirection::*;

    let direction = style.flex_direction;
    let is_row = matches!(direction, Row | RowReverse);
    let is_reverse = matches!(direction, RowReverse | ColumnReverse);

    let (available_main, available_cross) = if is_row {
        (constraints.width, constraints.height)
    } else {
        (constraints.height, constraints.width)
    };

    let available_main_cells = available_main.as_definite().unwrap_or(0);
    let available_cross_cells = available_cross.as_definite().unwrap_or(0);

    let (main_gap, cross_gap) = if is_row {
        (
            style.column_gap.resolve(available_main_cells),
            style.row_gap.resolve(available_cross_cells),
        )
    } else {
        (
            style.row_gap.resolve(available_main_cells),
            style.column_gap.resolve(available_cross_cells),
        )
    };

    let mut items = collect::collect_flex_items(
        document,
        node_id,
        direction,
        available_main,
        available_cross,
    );

    if is_reverse {
        items.reverse();
    }

    let mut lines =
        lines::collect_into_lines(&items, available_main_cells, style.flex_wrap, main_gap);

    for line in &mut lines {
        flexible::resolve_flexible_lengths(line, available_main_cells, main_gap);
    }

    cross::resolve_cross_axis(
        &mut lines,
        available_cross_cells,
        style.align_content,
        cross_gap,
    );

    align::justify_items(
        &mut lines,
        available_main_cells,
        style.justify_content,
        main_gap,
    );
    align::align_items(&mut lines, style.align_items);

    let total_main = lines
        .iter()
        .map(|line| align::line_main_size(line, main_gap))
        .max()
        .unwrap_or(0);
    let total_cross = cross::total_cross_size(&lines, cross_gap);

    for line in &lines {
        for item in &line.items {
            write_item(document, item, is_row);
        }
    }

    if is_row {
        Size::new(total_main, total_cross)
    } else {
        Size::new(total_cross, total_main)
    }
}

fn write_item<D: CapsuleDocument>(document: &mut D, item: &FlexItem<D::NodeId>, is_row: bool) {
    let (x, y) = if is_row {
        (item.main_position, item.cross_position)
    } else {
        (item.cross_position, item.main_position)
    };

    let (width, height) = if is_row {
        (item.main_size, item.cross_size)
    } else {
        (item.cross_size, item.main_size)
    };

    let node = document.get_node_mut(item.node_id);

    node.set_layout(Layout {
        order: 0,
        location: Point::new(x, y),
        scrollbar_size: Size::ZERO,
        resolved_box: ResolvedBox {
            content_size: Size::new(
                width
                    .saturating_sub(item.resolved_box.border.horizontal())
                    .saturating_sub(item.resolved_box.padding.horizontal()),
                height
                    .saturating_sub(item.resolved_box.border.horizontal())
                    .saturating_sub(item.resolved_box.padding.vertical()),
            ),
            ..item.resolved_box
        },
    });
}
