use crate::{
    FlexWrap,
    brief::flex::core::{FlexItem, FlexLine},
};

/// Collect flex items into lines based on wrap mode and available space.
pub fn collect_into_lines<NodeId: Copy>(
    items: &[FlexItem<NodeId>],
    available_main: u16,
    wrap: FlexWrap,
    gap: u16,
) -> Vec<FlexLine<NodeId>> {
    if items.is_empty() {
        return vec![];
    }

    match wrap {
        FlexWrap::NoWrap => collect_no_wrap(items),
        FlexWrap::Wrap => collect_wrap(items, available_main, gap),
        FlexWrap::WrapReverse => {
            let mut lines = collect_wrap(items, available_main, gap);
            lines.reverse();
            lines
        }
    }
}

fn collect_no_wrap<NodeId: Copy>(items: &[FlexItem<NodeId>]) -> Vec<FlexLine<NodeId>> {
    let cross_size = items
        .iter()
        .map(FlexItem::outer_cross_size)
        .max()
        .unwrap_or(0);

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
