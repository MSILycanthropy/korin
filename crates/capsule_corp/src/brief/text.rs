use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{Size, WhiteSpace, brief::core::AvailableSpace};

pub fn measure_text(text: &str, white_space: WhiteSpace, available_width: AvailableSpace) -> Size {
    match white_space {
        WhiteSpace::NoWrap | WhiteSpace::Pre => measure_no_wrap(text),
        WhiteSpace::Normal | WhiteSpace::PreWrap => match available_width {
            AvailableSpace::Definite(width) => measure_wrap(text, width),
            AvailableSpace::MinContent => measure_min_content(text),
            AvailableSpace::MaxContent => measure_no_wrap(text),
        },
    }
}

#[inline]
#[must_use]
fn measure_no_wrap(text: &str) -> Size {
    let width = u16::try_from(text.width()).unwrap_or(u16::MAX);
    let height = u16::from(!text.is_empty());
    Size::new(width, height)
}

#[must_use]
fn measure_wrap(text: &str, max_width: u16) -> Size {
    if text.is_empty() || max_width == 0 {
        return Size::ZERO;
    }

    let mut lines = 1u16;
    let mut current_width = 0u16;
    let mut max_line_width = 0u16;

    for segment in text.split_word_bounds() {
        let is_whitespace = segment.trim().is_empty();

        let segment_width = u16::try_from(segment.width()).unwrap_or(u16::MAX);

        if current_width == 0 && is_whitespace {
            continue;
        }

        if current_width + segment_width > max_width {
            if is_whitespace {
                max_line_width = max_line_width.max(current_width);
                lines = lines.saturating_add(1);
                current_width = 0;
                continue;
            }

            if current_width > 0 {
                max_line_width = max_line_width.max(current_width);
                lines = lines.saturating_add(1);
            }

            current_width = segment_width;
        } else {
            current_width += segment_width;
        }
    }

    max_line_width = max_line_width.max(current_width);
    Size::new(max_line_width, lines)
}

#[must_use]
fn measure_min_content(text: &str) -> Size {
    let max_word_width = text
        .split_word_bounds()
        .map(|word| u16::try_from(word.width()).unwrap_or(u16::MAX))
        .max()
        .unwrap_or(0);

    let height = u16::from(!text.is_empty());
    Size::new(max_word_width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wrap_simple() {
        let size = measure_text("hello", WhiteSpace::NoWrap, AvailableSpace::Definite(100));
        assert_eq!(size, Size::new(5, 1));
    }

    #[test]
    fn no_wrap_empty() {
        let size = measure_text("", WhiteSpace::NoWrap, AvailableSpace::Definite(100));
        assert_eq!(size, Size::new(0, 0));
    }

    #[test]
    fn no_wrap_wide_chars() {
        // CJK characters are 2 cells wide
        let size = measure_text("日本語", WhiteSpace::NoWrap, AvailableSpace::Definite(100));
        assert_eq!(size, Size::new(6, 1));
    }

    #[test]
    fn wrap_fits() {
        let size = measure_text(
            "hello world",
            WhiteSpace::Normal,
            AvailableSpace::Definite(20),
        );
        assert_eq!(size, Size::new(11, 1));
    }

    #[test]
    fn wrap_breaks() {
        let size = measure_text(
            "hello world",
            WhiteSpace::Normal,
            AvailableSpace::Definite(8),
        );
        assert_eq!(size, Size::new(6, 2)); // "hello " / "world"
    }

    #[test]
    fn wrap_multiple_lines() {
        let size = measure_text(
            "one two three four",
            WhiteSpace::Normal,
            AvailableSpace::Definite(9),
        );
        // "one two" (7) / "three" (5) / "four" (4)
        assert_eq!(size.height, 3);
    }

    #[test]
    fn min_content() {
        let size = measure_text(
            "hello wonderful world",
            WhiteSpace::Normal,
            AvailableSpace::MinContent,
        );
        assert_eq!(size.width, 9);
    }

    #[test]
    fn max_content() {
        let size = measure_text(
            "hello world",
            WhiteSpace::Normal,
            AvailableSpace::MaxContent,
        );
        assert_eq!(size, Size::new(11, 1)); // no wrap
    }

    #[test]
    fn cjk_words() {
        let size = measure_text(
            "日本語テスト",
            WhiteSpace::Normal,
            AvailableSpace::MinContent,
        );
        assert_eq!(size.width, 6);
    }
}
