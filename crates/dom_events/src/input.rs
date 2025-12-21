/// The type of input operation.
///
/// Specification: <https://w3c.github.io/input-events/#interface-InputEvent>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
    /// Insert typed text.
    InsertText,
    /// Replace existing text via IME.
    InsertReplacementText,
    /// Insert a line break.
    InsertLineBreak,
    /// Insert a paragraph break.
    InsertParagraph,
    /// Insert an ordered list.
    InsertOrderedList,
    /// Insert an unordered list.
    InsertUnorderedList,
    /// Insert a horizontal rule.
    InsertHorizontalRule,
    /// Insert from paste.
    InsertFromPaste,
    /// Insert from drop.
    InsertFromDrop,
    /// Insert from yank (kill buffer).
    InsertFromYank,
    /// Insert via IME.
    InsertCompositionText,
    /// Insert a link.
    InsertLink,
    /// Delete text before cursor (backspace).
    DeleteContentBackward,
    /// Delete text after cursor (delete).
    DeleteContentForward,
    /// Delete by word before cursor.
    DeleteWordBackward,
    /// Delete by word after cursor.
    DeleteWordForward,
    /// Delete by soft line before cursor.
    DeleteSoftLineBackward,
    /// Delete by soft line after cursor.
    DeleteSoftLineForward,
    /// Delete by hard line before cursor.
    DeleteHardLineBackward,
    /// Delete by hard line after cursor.
    DeleteHardLineForward,
    /// Delete by cut.
    DeleteByCut,
    /// Delete by drag.
    DeleteByDrag,
    /// Delete entire content.
    DeleteContent,
    /// Undo.
    HistoryUndo,
    /// Redo.
    HistoryRedo,
    /// Bold.
    FormatBold,
    /// Italic.
    FormatItalic,
    /// Underline.
    FormatUnderline,
    /// Strikethrough.
    FormatStrikeThrough,
    /// Superscript.
    FormatSuperscript,
    /// Subscript.
    FormatSubscript,
    /// Justify full.
    FormatJustifyFull,
    /// Justify center.
    FormatJustifyCenter,
    /// Justify right.
    FormatJustifyRight,
    /// Justify left.
    FormatJustifyLeft,
    /// Indent.
    FormatIndent,
    /// Outdent.
    FormatOutdent,
    /// Remove formatting.
    FormatRemove,
    /// Set block text direction.
    FormatSetBlockTextDirection,
    /// Set inline text direction.
    FormatSetInlineTextDirection,
    /// Set background color.
    FormatBackColor,
    /// Set font color.
    FormatFontColor,
    /// Set font name.
    FormatFontName,
}

/// Input event data.
///
/// Specification: <https://w3c.github.io/uievents/#interface-inputevent>
#[derive(Clone, Debug)]
pub struct InputEvent {
    /// Inserted text, if any.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-inputevent-data>
    pub data: Option<String>,
    /// Whether this is part of an IME composition.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-inputevent-iscomposing>
    pub is_composing: bool,
    /// The type of input.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-inputevent-inputtype>
    pub input_type: InputType,
}
