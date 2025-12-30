use crate::{AlignContent, brief::flex::core::FlexLine};

/// Resolve cross-axis sizes and line positions.
///
/// - Computes each line's cross size (already done in lines.rs, but can be adjusted)
/// - Distributes lines along cross axis based on `align-content`
/// - Sets `cross_position` for each line
#[allow(clippy::cast_possible_truncation)]
pub fn resolve_cross_axis<NodeId: Copy>(
    lines: &mut [FlexLine<NodeId>],
    available_cross: u16,
    align_content: AlignContent,
    gap: u16,
) {
    pub use AlignContent::*;

    if lines.is_empty() {
        return;
    }

    let total_lines_cross: u16 = lines.iter().map(|line| line.cross_size).sum();
    let total_gaps = gap.saturating_mul(lines.len().saturating_sub(1) as u16);
    let free_space = available_cross
        .saturating_sub(total_lines_cross)
        .saturating_sub(total_gaps);

    let (start_offset, between_space) = match align_content {
        FlexStart | Start => (0, gap),
        FlexEnd | End => (free_space, gap),
        Center => (free_space / 2, gap),
        SpaceBetween => {
            if lines.len() == 1 {
                (0, 0)
            } else {
                let between = free_space / (lines.len() - 1) as u16;
                (0, between.saturating_add(gap))
            }
        }
        SpaceAround => {
            if lines.len() == 1 {
                (free_space / 2, 0)
            } else {
                let space_per_item = free_space / lines.len() as u16;
                (space_per_item / 2, space_per_item.saturating_add(gap))
            }
        }
        SpaceEvenly => {
            let slots = lines.len() + 1;
            let space = free_space / slots as u16;

            (space, space.saturating_add(gap))
        }
        Stretch => {
            if free_space > 0 && !lines.is_empty() {
                let extra_per_line = free_space / lines.len() as u16;

                for line in lines.iter_mut() {
                    line.cross_size = line.cross_size.saturating_add(extra_per_line);
                }
            }

            (0, gap)
        }
    };

    let mut cross_position = start_offset;
    for line in lines.iter_mut() {
        line.cross_position = cross_position;
        cross_position = cross_position
            .saturating_add(line.cross_size)
            .saturating_add(between_space);
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn total_cross_size<NodeId: Copy>(lines: &[FlexLine<NodeId>], gap: u16) -> u16 {
    if lines.is_empty() {
        return 0;
    }

    let lines_size: u16 = lines.iter().map(|line| line.cross_size).sum();
    let gaps = gap.saturating_mul(lines.len().saturating_sub(1) as u16);

    lines_size.saturating_add(gaps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_line<NodeId: Copy + Default>(cross_size: u16) -> FlexLine<NodeId> {
        FlexLine {
            items: vec![],
            cross_size,
            cross_position: 0,
        }
    }

    #[test]
    fn flex_start_positions_at_zero() {
        let mut lines = vec![make_line::<usize>(20), make_line(30)];

        resolve_cross_axis(&mut lines, 100, AlignContent::FlexStart, 0);

        assert_eq!(lines[0].cross_position, 0);
        assert_eq!(lines[1].cross_position, 20);
    }

    #[test]
    fn flex_end_positions_at_end() {
        let mut lines = vec![make_line::<usize>(20), make_line(30)];

        // Total: 50, available: 100, free: 50
        resolve_cross_axis(&mut lines, 100, AlignContent::FlexEnd, 0);

        assert_eq!(lines[0].cross_position, 50);
        assert_eq!(lines[1].cross_position, 70);
    }

    #[test]
    fn center_positions_in_middle() {
        let mut lines = vec![make_line::<usize>(20), make_line(30)];

        // Total: 50, available: 100, free: 50, offset: 25
        resolve_cross_axis(&mut lines, 100, AlignContent::Center, 0);

        assert_eq!(lines[0].cross_position, 25);
        assert_eq!(lines[1].cross_position, 45);
    }

    #[test]
    fn space_between_distributes_evenly() {
        let mut lines = vec![make_line::<usize>(20), make_line(20), make_line(20)];

        // Total: 60, available: 100, free: 40
        // 2 gaps, 20 each
        resolve_cross_axis(&mut lines, 100, AlignContent::SpaceBetween, 0);

        assert_eq!(lines[0].cross_position, 0);
        assert_eq!(lines[1].cross_position, 40); // 20 + 20
        assert_eq!(lines[2].cross_position, 80); // 40 + 20 + 20
    }

    #[test]
    fn stretch_expands_lines() {
        let mut lines = vec![make_line::<usize>(20), make_line(20)];

        // Total: 40, available: 100, free: 60
        // Each line gets 30 extra
        resolve_cross_axis(&mut lines, 100, AlignContent::Stretch, 0);

        assert_eq!(lines[0].cross_size, 50);
        assert_eq!(lines[1].cross_size, 50);
        assert_eq!(lines[0].cross_position, 0);
        assert_eq!(lines[1].cross_position, 50);
    }

    #[test]
    fn gap_adds_space_between_lines() {
        let mut lines = vec![make_line::<usize>(20), make_line(20)];

        resolve_cross_axis(&mut lines, 100, AlignContent::FlexStart, 10);

        assert_eq!(lines[0].cross_position, 0);
        assert_eq!(lines[1].cross_position, 30); // 20 + 10 gap
    }

    #[test]
    fn total_cross_size_sums_lines_and_gaps() {
        let lines = vec![make_line::<usize>(20), make_line(30), make_line(25)];

        assert_eq!(total_cross_size(&lines, 0), 75);
        assert_eq!(total_cross_size(&lines, 5), 85); // 75 + 2*5
    }

    #[test]
    fn empty_lines_returns_zero() {
        let lines: Vec<FlexLine<usize>> = vec![];
        assert_eq!(total_cross_size(&lines, 10), 0);
    }
}
