use crate::brief::flex::core::{FlexItem, FlexLine};

/// Resolve flexible lengths for a single line.
///
/// Distributes free space among items based on flex-grow,
/// or shrinks items based on flex-shrink if overflowing.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn resolve_flexible_lengths<NodeId: Copy>(
    line: &mut FlexLine<NodeId>,
    available_main: u16,
    gap: u16,
) {
    if line.items.is_empty() {
        return;
    }

    let total_gap = gap.saturating_mul(line.items.len().saturating_sub(1) as u16);
    let available_for_items = available_main.saturating_sub(total_gap);

    let total_hypothetical: u16 = line
        .items
        .iter()
        .map(FlexItem::outer_hypothetical_main_size)
        .sum();

    let free_space = i32::from(available_for_items) - i32::from(total_hypothetical);

    if free_space.is_positive() {
        grow_items(line, free_space as u16);
    } else if free_space.is_negative() {
        shrink_items(line, free_space.unsigned_abs() as u16);
    }
}

/// Distribute positive free space among items with flex-grow > 0.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn grow_items<NodeId: Copy>(line: &mut FlexLine<NodeId>, mut free_space: u16) {
    loop {
        let total_grow: f32 = line
            .items
            .iter()
            .filter(|item| !item.frozen)
            .map(|item| item.flex_grow)
            .sum();

        if total_grow <= 0.0 || free_space == 0 {
            for item in &mut line.items {
                if !item.frozen {
                    item.main_size = item.hypothetical_main_size;
                    item.frozen = true;
                }
            }
            break;
        }

        let space_per_grow = f32::from(free_space) / total_grow;
        let mut any_clamped = false;

        for item in &mut line.items {
            if item.frozen {
                continue;
            }

            let grow_amount = (item.flex_grow * space_per_grow).floor() as u16;
            let new_size = item.hypothetical_main_size.saturating_add(grow_amount);

            if let Some(max) = item.max_main_size
                && new_size >= max
            {
                item.main_size = max;
                item.frozen = true;

                let used = max.saturating_sub(item.hypothetical_main_size);
                free_space = free_space.saturating_sub(used);
                any_clamped = true;

                continue;
            }

            item.main_size = new_size;
        }

        if !any_clamped {
            for item in &mut line.items {
                if !item.frozen {
                    item.frozen = true;
                }
            }
            return;
        }
    }
}

/// Remove overflow from items with flex-shrink > 0.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn shrink_items<NodeId: Copy>(line: &mut FlexLine<NodeId>, mut overflow: u16) {
    loop {
        let total_shrink: f32 = line
            .items
            .iter()
            .filter(|i| !i.frozen)
            .map(|i| i.flex_shrink * f32::from(i.flex_basis))
            .sum();

        if total_shrink <= 0.0 || overflow == 0 {
            for item in &mut line.items {
                if !item.frozen {
                    item.main_size = item.hypothetical_main_size;
                    item.frozen = true;
                }
            }
            break;
        }

        let mut any_clamped = false;

        for item in &mut line.items {
            if item.frozen {
                continue;
            }

            let scaled_shrink = item.flex_shrink * f32::from(item.flex_basis);
            let shrink_ratio = scaled_shrink / total_shrink;
            let shrink_amount = (f32::from(overflow) * shrink_ratio).floor() as u16;
            let new_size = item.hypothetical_main_size.saturating_sub(shrink_amount);

            if new_size <= item.min_main_size {
                item.main_size = item.min_main_size;
                item.frozen = true;

                let shrunk = item
                    .hypothetical_main_size
                    .saturating_sub(item.min_main_size);
                overflow = overflow.saturating_sub(shrunk);
                any_clamped = true;

                continue;
            }

            item.main_size = new_size;
        }

        if !any_clamped {
            for item in &mut line.items {
                if !item.frozen {
                    item.frozen = true;
                }
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Edges;
    use crate::brief::box_model::ResolvedBox;

    fn make_item(hypothetical_main_size: u16, grow: f32, shrink: f32) -> FlexItem<usize> {
        FlexItem {
            node_id: 0,
            align_self: crate::AlignSelf::Auto,
            flex_grow: grow,
            flex_shrink: shrink,
            flex_basis: hypothetical_main_size,
            min_main_size: 0,
            max_main_size: None,
            hypothetical_main_size,
            margin: Edges::ZERO,
            resolved_box: ResolvedBox::ZERO,
            frozen: false,
            main_size: hypothetical_main_size,
            cross_size: 10,
            main_position: 0,
            cross_position: 0,
        }
    }

    fn make_line(items: Vec<FlexItem<usize>>) -> FlexLine<usize> {
        FlexLine {
            items,
            cross_size: 10,
            cross_position: 0,
        }
    }

    #[test]
    fn no_grow_when_no_free_space() {
        let mut line = make_line(vec![make_item(50, 1.0, 1.0), make_item(50, 1.0, 1.0)]);

        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 50);
        assert_eq!(line.items[1].main_size, 50);
    }

    #[test]
    fn grow_distributes_free_space() {
        let mut line = make_line(vec![make_item(20, 1.0, 1.0), make_item(20, 1.0, 1.0)]);

        // Available 100, items take 40, free space = 60
        // Each has flex-grow 1, so each gets 30
        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 50);
        assert_eq!(line.items[1].main_size, 50);
    }

    #[test]
    fn grow_respects_ratio() {
        let mut line = make_line(vec![make_item(20, 1.0, 1.0), make_item(20, 3.0, 1.0)]);

        // Free space = 60, total grow = 4
        // Item 0: 60 * (1/4) = 15 → 20 + 15 = 35
        // Item 1: 60 * (3/4) = 45 → 20 + 45 = 65
        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 35);
        assert_eq!(line.items[1].main_size, 65);
    }

    #[test]
    fn grow_zero_no_growth() {
        let mut line = make_line(vec![make_item(20, 0.0, 1.0), make_item(20, 1.0, 1.0)]);

        // Free space = 60, only item 1 grows
        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 20);
        assert_eq!(line.items[1].main_size, 80);
    }

    #[test]
    fn shrink_removes_overflow() {
        let mut line = make_line(vec![make_item(60, 1.0, 1.0), make_item(60, 1.0, 1.0)]);

        // Available 100, items take 120, overflow = 20
        // Both have same flex-shrink * flex-basis, so shrink equally
        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 50);
        assert_eq!(line.items[1].main_size, 50);
    }

    #[test]
    fn shrink_respects_min() {
        let mut line = make_line(vec![
            {
                let mut item = make_item(60, 1.0, 1.0);
                item.min_main_size = 55;
                item
            },
            make_item(60, 1.0, 1.0),
        ]);

        // Overflow = 20
        // First pass: each would shrink 10, but item 0 hits min (55)
        //   item 0: frozen at 55, used 5 of the overflow
        //   remaining overflow: 20 - 5 = 15
        // Second pass: item 1 shrinks by remaining 15
        //   item 1: 60 - 15 = 45
        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 55);
        assert_eq!(line.items[1].main_size, 45);
    }

    #[test]
    fn grow_respects_max() {
        let mut line = make_line(vec![
            {
                let mut item = make_item(20, 1.0, 1.0);
                item.max_main_size = Some(30);
                item
            },
            make_item(20, 1.0, 1.0),
        ]);

        // Free space = 60
        // First pass: each would grow 30, but item 0 hits max (30)
        //   item 0: frozen at 30, used 10 of the free space
        //   remaining free space: 60 - 10 = 50
        // Second pass: item 1 grows by remaining 50
        //   item 1: 20 + 50 = 70
        resolve_flexible_lengths(&mut line, 100, 0);

        assert_eq!(line.items[0].main_size, 30);
        assert_eq!(line.items[1].main_size, 70);
    }

    #[test]
    fn gap_reduces_available_space() {
        let mut line = make_line(vec![make_item(20, 1.0, 1.0), make_item(20, 1.0, 1.0)]);

        // Available 100, gap 10, available for items = 90
        // Items take 40, free space = 50
        resolve_flexible_lengths(&mut line, 100, 10);

        assert_eq!(line.items[0].main_size, 45);
        assert_eq!(line.items[1].main_size, 45);
    }
}
