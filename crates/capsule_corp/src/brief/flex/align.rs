use crate::{
    AlignItems, JustifyContent,
    brief::flex::core::{FlexItem, FlexLine},
};

/// Position items along main axis within each line.
///
/// Applies `justify-content` and `column-gap`.
pub fn justify_items<NodeId: Copy>(
    lines: &mut [FlexLine<NodeId>],
    available_main: u16,
    justify_content: JustifyContent,
    gap: u16,
) {
    for line in lines {
        justify_line(line, available_main, justify_content, gap);
    }
}

/// Position items along main axis for a single line.
#[allow(clippy::cast_possible_truncation)]
fn justify_line<NodeId: Copy>(
    line: &mut FlexLine<NodeId>,
    available_main: u16,
    justify_content: JustifyContent,
    gap: u16,
) {
    if line.items.is_empty() {
        return;
    }

    let total_items_main: u16 = line.items.iter().map(FlexItem::outer_main_size).sum();
    let total_gaps = gap.saturating_mul(line.items.len().saturating_sub(1) as u16);

    let free_space = available_main
        .saturating_sub(total_items_main)
        .saturating_sub(total_gaps);

    let (start_offset, between_space) = match justify_content {
        JustifyContent::FlexStart | JustifyContent::Start | JustifyContent::Stretch => (0, gap),
        JustifyContent::FlexEnd | JustifyContent::End => (free_space, gap),
        JustifyContent::Center => (free_space / 2, gap),
        JustifyContent::SpaceBetween => {
            if line.items.len() == 1 {
                (0, 0)
            } else {
                let between = free_space / (line.items.len() - 1) as u16;
                (0, between.saturating_add(gap))
            }
        }
        JustifyContent::SpaceAround => {
            if line.items.len() == 1 {
                (free_space / 2, 0)
            } else {
                let space_per_item = free_space / line.items.len() as u16;
                (space_per_item / 2, space_per_item.saturating_add(gap))
            }
        }
        JustifyContent::SpaceEvenly => {
            let slots = line.items.len() + 1;
            let space = free_space / slots as u16;
            (space, space.saturating_add(gap))
        }
    };

    let mut main_position = start_offset;
    for item in &mut line.items {
        item.main_position = main_position.saturating_add(item.margin.left);
        main_position = main_position
            .saturating_add(item.outer_main_size())
            .saturating_add(between_space);
    }
}

/// Position items along cross axis within each line.
///
/// Applies `align-items` (container default) and `align-self` (per-item override).
pub fn align_items<NodeId: Copy>(lines: &mut [FlexLine<NodeId>], align_items: AlignItems) {
    for line in lines {
        align_line(line, align_items);
    }
}

/// Position items along cross axis for a single line.
fn align_line<NodeId: Copy>(line: &mut FlexLine<NodeId>, default_align: AlignItems) {
    use AlignItems::*;

    let line_cross_size = line.cross_size;
    let line_cross_position = line.cross_position;

    for item in &mut line.items {
        let align = item.align_self.resolve(default_align);

        let item_outer_cross = item.outer_cross_size();
        let free_space = line_cross_size.saturating_sub(item_outer_cross);

        let cross_offset = match align {
            FlexStart | Start | Baseline => 0,
            FlexEnd | End => free_space,
            Center => free_space / 2,
            Stretch => {
                item.cross_size = line_cross_size
                    .saturating_sub(item.margin.top)
                    .saturating_sub(item.margin.bottom);
                0
            }
        };

        item.cross_position = line_cross_position
            .saturating_add(cross_offset)
            .saturating_add(item.margin.top);
    }
}

/// Total main size used by a line (including gaps).
#[allow(clippy::cast_possible_truncation)]
pub fn line_main_size<NodeId: Copy>(line: &FlexLine<NodeId>, gap: u16) -> u16 {
    if line.items.is_empty() {
        return 0;
    }

    let items_size: u16 = line.items.iter().map(FlexItem::outer_main_size).sum();
    let gaps = gap.saturating_mul(line.items.len().saturating_sub(1) as u16);

    items_size.saturating_add(gaps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlignSelf, Edges, brief::box_model::ResolvedBox};

    fn make_item(node_id: usize, main_size: u16, cross_size: u16) -> FlexItem<usize> {
        FlexItem {
            node_id,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_basis: main_size,
            min_main_size: 0,
            max_main_size: None,
            hypothetical_main_size: main_size,
            margin: Edges::ZERO,
            resolved_box: ResolvedBox::ZERO,
            align_self: AlignSelf::Auto,
            frozen: true,
            main_size,
            cross_size,
            main_position: 0,
            cross_position: 0,
        }
    }

    fn make_line(items: Vec<FlexItem<usize>>, cross_size: u16) -> FlexLine<usize> {
        FlexLine {
            items,
            cross_size,
            cross_position: 0,
        }
    }

    #[test]
    fn justify_flex_start() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 30, 10)],
            10,
        )];

        justify_items(&mut lines, 100, JustifyContent::FlexStart, 0);

        assert_eq!(lines[0].items[0].main_position, 0);
        assert_eq!(lines[0].items[1].main_position, 20);
    }

    #[test]
    fn justify_flex_end() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 30, 10)],
            10,
        )];

        // Items: 50, available: 100, free: 50
        justify_items(&mut lines, 100, JustifyContent::FlexEnd, 0);

        assert_eq!(lines[0].items[0].main_position, 50);
        assert_eq!(lines[0].items[1].main_position, 70);
    }

    #[test]
    fn justify_center() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 30, 10)],
            10,
        )];

        // Free: 50, offset: 25
        justify_items(&mut lines, 100, JustifyContent::Center, 0);

        assert_eq!(lines[0].items[0].main_position, 25);
        assert_eq!(lines[0].items[1].main_position, 45);
    }

    #[test]
    fn justify_space_between() {
        let mut lines = vec![make_line(
            vec![
                make_item(0, 20, 10),
                make_item(1, 20, 10),
                make_item(2, 20, 10),
            ],
            10,
        )];

        // Items: 60, free: 40, 2 gaps of 20
        justify_items(&mut lines, 100, JustifyContent::SpaceBetween, 0);

        assert_eq!(lines[0].items[0].main_position, 0);
        assert_eq!(lines[0].items[1].main_position, 40);
        assert_eq!(lines[0].items[2].main_position, 80);
    }

    #[test]
    fn justify_with_gap() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 20, 10)],
            10,
        )];

        justify_items(&mut lines, 100, JustifyContent::FlexStart, 10);

        assert_eq!(lines[0].items[0].main_position, 0);
        assert_eq!(lines[0].items[1].main_position, 30); // 20 + 10 gap
    }

    #[test]
    fn align_flex_start() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 20, 15)],
            20, // line cross size
        )];
        lines[0].cross_position = 5;

        align_items(&mut lines, AlignItems::FlexStart);

        assert_eq!(lines[0].items[0].cross_position, 5);
        assert_eq!(lines[0].items[1].cross_position, 5);
    }

    #[test]
    fn align_flex_end() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 20, 15)],
            20,
        )];
        lines[0].cross_position = 0;

        align_items(&mut lines, AlignItems::FlexEnd);

        // Item 0: cross_size 10, line 20, free 10 -> offset 10
        // Item 1: cross_size 15, line 20, free 5 -> offset 5
        assert_eq!(lines[0].items[0].cross_position, 10);
        assert_eq!(lines[0].items[1].cross_position, 5);
    }

    #[test]
    fn align_center() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 20, 20)],
            20,
        )];
        lines[0].cross_position = 0;

        align_items(&mut lines, AlignItems::Center);

        // Item 0: free 10, offset 5
        // Item 1: free 0, offset 0
        assert_eq!(lines[0].items[0].cross_position, 5);
        assert_eq!(lines[0].items[1].cross_position, 0);
    }

    #[test]
    fn align_stretch() {
        let mut lines = vec![make_line(
            vec![make_item(0, 20, 10), make_item(1, 20, 15)],
            30,
        )];
        lines[0].cross_position = 0;

        align_items(&mut lines, AlignItems::Stretch);

        // Both items stretched to line height
        assert_eq!(lines[0].items[0].cross_size, 30);
        assert_eq!(lines[0].items[1].cross_size, 30);
        assert_eq!(lines[0].items[0].cross_position, 0);
        assert_eq!(lines[0].items[1].cross_position, 0);
    }

    #[test]
    fn line_main_size_with_gaps() {
        let line = make_line(
            vec![
                make_item(0, 20, 10),
                make_item(1, 30, 10),
                make_item(2, 25, 10),
            ],
            10,
        );

        assert_eq!(line_main_size(&line, 0), 75);
        assert_eq!(line_main_size(&line, 5), 85); // 75 + 2*5
    }
}
