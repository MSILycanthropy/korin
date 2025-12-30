use crate::{
    FlexWrap,
    brief::flex::core::{FlexItem, FlexLine},
};

/// Collect flex items into lines based on wrap mode and available space.
pub fn collect_into_lines<NodeId: Copy>(
    items: &[FlexItem<NodeId>],
    available_main: u16,
    available_cross: u16,
    wrap: FlexWrap,
    gap: u16,
) -> Vec<FlexLine<NodeId>> {
    if items.is_empty() {
        return vec![];
    }

    match wrap {
        FlexWrap::NoWrap => collect_no_wrap(items, available_cross),
        FlexWrap::Wrap => collect_wrap(items, available_main, gap),
        FlexWrap::WrapReverse => {
            let mut lines = collect_wrap(items, available_main, gap);
            lines.reverse();
            lines
        }
    }
}

fn collect_no_wrap<NodeId: Copy>(
    items: &[FlexItem<NodeId>],
    available_cross: u16,
) -> Vec<FlexLine<NodeId>> {
    let cross_size = items
        .iter()
        .map(FlexItem::outer_cross_size)
        .max()
        .unwrap_or(0)
        .max(available_cross);

    vec![FlexLine {
        items: items.to_vec(),
        cross_size,
        cross_position: 0,
    }]
}

fn collect_wrap<NodeId: Copy>(
    items: &[FlexItem<NodeId>],
    available_main: u16,
    gap: u16,
) -> Vec<FlexLine<NodeId>> {
    let mut lines = vec![];
    let mut current_line = FlexLine::new();
    let mut line_main_size = 0u16;

    for item in items {
        let item_main_size = item.outer_hypothetical_main_size();

        let needed_space = if current_line.items.is_empty() {
            item_main_size
        } else {
            item_main_size.saturating_add(gap)
        };

        let fits = line_main_size.saturating_add(needed_space) <= available_main;

        if !fits && !current_line.items.is_empty() {
            finalize_line(&mut current_line);

            lines.push(current_line);

            current_line = FlexLine::new();
            line_main_size = 0;
        }

        if !current_line.items.is_empty() {
            line_main_size = line_main_size.saturating_add(gap);
        }

        line_main_size = line_main_size.saturating_add(item_main_size);
        current_line.items.push(*item);
    }

    if !current_line.items.is_empty() {
        finalize_line(&mut current_line);
        lines.push(current_line);
    }

    lines
}

fn finalize_line<NodeId: Copy>(line: &mut FlexLine<NodeId>) {
    line.cross_size = line
        .items
        .iter()
        .map(FlexItem::outer_cross_size)
        .max()
        .unwrap_or(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edges;
    use crate::brief::box_model::ResolvedBox;

    fn make_item(id: usize, hypothetical_main_size: u16, cross: u16) -> FlexItem<usize> {
        FlexItem {
            node_id: id,
            align_self: crate::AlignSelf::Auto,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_basis: hypothetical_main_size,
            min_main_size: 0,
            max_main_size: None,
            hypothetical_main_size,
            margin: Edges::ZERO,
            resolved_box: ResolvedBox::ZERO,
            frozen: false,
            main_size: hypothetical_main_size,
            cross_size: cross,
            main_position: 0,
            cross_position: 0,
        }
    }

    #[test]
    fn no_wrap_single_line() {
        let items = vec![
            make_item(0, 30, 10),
            make_item(1, 30, 15),
            make_item(2, 30, 10),
        ];

        let lines = collect_into_lines(&items, 50, 15, FlexWrap::NoWrap, 0);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].items.len(), 3);
        assert_eq!(lines[0].cross_size, 15);
    }

    #[test]
    fn no_wrap_uses_available_cross() {
        let items = vec![make_item(0, 30, 10)];

        let lines = collect_into_lines(&items, 100, 50, FlexWrap::NoWrap, 0);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].cross_size, 50); // Uses available_cross, not item's 10
    }

    #[test]
    fn wrap_creates_multiple_lines() {
        let items = vec![
            make_item(0, 30, 10),
            make_item(1, 30, 15),
            make_item(2, 30, 12),
        ];

        let lines = collect_into_lines(&items, 50, 100, FlexWrap::Wrap, 0);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].items.len(), 1);
        assert_eq!(lines[1].items.len(), 1);
        assert_eq!(lines[2].items.len(), 1);
    }

    #[test]
    fn wrap_with_gap() {
        let items = vec![
            make_item(0, 20, 10),
            make_item(1, 20, 10),
            make_item(2, 20, 10),
        ];

        // Available 50, items 20 each, gap 5
        // First two: 20 + 5 + 20 = 45 ✓
        // Third: 45 + 5 + 20 = 70 ✗ -> new line
        let lines = collect_into_lines(&items, 50, 100, FlexWrap::Wrap, 5);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].items.len(), 2);
        assert_eq!(lines[1].items.len(), 1);
    }

    #[test]
    fn wrap_reverse_reverses_lines() {
        let items = vec![make_item(0, 30, 10), make_item(1, 30, 10)];

        let lines = collect_into_lines(&items, 30, 100, FlexWrap::WrapReverse, 0);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].items[0].node_id, 1);
        assert_eq!(lines[1].items[0].node_id, 0);
    }

    #[test]
    fn empty_items_returns_empty() {
        let lines = collect_into_lines::<usize>(&[], 100, 100, FlexWrap::Wrap, 0);
        assert!(lines.is_empty());
    }

    #[test]
    fn single_item_too_large_still_placed() {
        let items = vec![make_item(0, 100, 10)];

        let lines = collect_into_lines(&items, 50, 100, FlexWrap::Wrap, 0);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].items.len(), 1);
    }
}
