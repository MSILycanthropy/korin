use crate::{
    AlignItems, AlignSelf, BorderStyle, Color, CustomValue, Dimension, Display, FlexDirection,
    FlexWrap, FontStyle, FontWeight, JustifyContent, Length, Overflow, OverflowWrap, TextAlign,
    TextDecoration, UnresolvedValue, VerticalAlign, Visibility, WhiteSpace, macros::keyword_enum,
};

keyword_enum! {
    #[derive(Copy)]
    pub enum Property {
        Display = "display",

        FlexDirection = "flex-direction",
        FlexWrap = "flex-wrap",
        JustifyContent = "justify-content",
        AlignItems = "align-items",

        FlexGrow = "flex-grow",
        FlexShrink = "flex-shrink",
        FlexBasis = "flex-basis",
        AlignSelf = "align-self",

        GridTemplateColumns = "grid-template-columns",
        GridTemplateRows = "grid-template-rows",

        GridColumn = "grid-column",
        GridRow = "grid-row",

        RowGap = "row-gap",
        ColumnGap = "column-gap",

        Width = "width",
        Height = "height",
        MinWidth = "min-width",
        MaxWidth = "max-width",
        MinHeight = "min-height",
        MaxHeight = "max-height",

        MarginTop = "margin-top",
        MarginRight = "margin-right",
        MarginBottom = "margin-bottom",
        MarginLeft = "margin-left",

        PaddingTop = "padding-top",
        PaddingRight = "padding-right",
        PaddingBottom = "padding-bottom",
        PaddingLeft = "padding-left",

        BorderTopStyle = "border-top-style",
        BorderRightStyle = "border-right-style",
        BorderBottomStyle = "border-bottom-style",
        BorderLeftStyle = "border-left-style",
        BorderTopColor = "border-top-color",
        BorderRightColor = "border-right-color",
        BorderBottomColor = "border-bottom-color",
        BorderLeftColor = "border-left-color",

        Color = "color",
        BackgroundColor = "background-color",

        FontWeight = "font-weight",
        FontStyle = "font-style",
        TextDecoration = "text-decoration",
        TextAlign = "text-align",
        VerticalAlign = "vertical-align",
        WhiteSpace = "white-space",
        OverflowWrap = "overflow-wrap",

        OverflowX = "overflow-x",
        OverflowY = "overflow-y",
        Visibility = "visibility",

        ZIndex = "z-index",

        @custom
    }
}

impl Property {
    #[must_use]
    pub const fn inherited(&self) -> bool {
        use Property::*;

        matches!(
            self,
            Color
                | FontWeight
                | FontStyle
                | TextDecoration
                | TextAlign
                | WhiteSpace
                | OverflowWrap
                | Visibility
        )
    }
}

keyword_enum! {
    pub enum Shorthand {
        Margin = "margin",
        Padding = "padding",
        Border = "border",
        BorderStyle = "border-style",
        BorderColor = "border-color",
        BorderTop = "border-top",
        BorderRight = "border-right",
        BorderBottom = "border-bottom",
        BorderLeft = "border-left",
        Flex = "flex",
        Gap = "gap",
        Overflow = "overflow",
        Background = "background",
    }
}

impl Shorthand {
    #[must_use]
    pub const fn longhands(&self) -> &'static [Property] {
        use Property::*;
        match self {
            Self::Margin => &[MarginTop, MarginRight, MarginBottom, MarginLeft],
            Self::Padding => &[PaddingTop, PaddingRight, PaddingBottom, PaddingLeft],
            Self::Border => &[
                BorderTopStyle,
                BorderRightStyle,
                BorderBottomStyle,
                BorderLeftStyle,
                BorderTopColor,
                BorderRightColor,
                BorderBottomColor,
                BorderLeftColor,
            ],
            Self::BorderStyle => &[
                BorderTopStyle,
                BorderRightStyle,
                BorderBottomStyle,
                BorderLeftStyle,
            ],
            Self::BorderColor => &[
                BorderTopColor,
                BorderRightColor,
                BorderBottomColor,
                BorderLeftColor,
            ],
            Self::BorderTop => &[BorderTopStyle, BorderTopColor],
            Self::BorderRight => &[BorderRightStyle, BorderRightColor],
            Self::BorderBottom => &[BorderBottomStyle, BorderBottomColor],
            Self::BorderLeft => &[BorderLeftStyle, BorderLeftColor],
            Self::Flex => &[FlexGrow, FlexShrink, FlexBasis],
            Self::Gap => &[RowGap, ColumnGap],
            Self::Overflow => &[OverflowX, OverflowY],
            Self::Background => &[BackgroundColor],
        }
    }
}

#[derive(Clone, Copy)]
pub enum PropertyName {
    Longhand(Property),
    Shorthand(Shorthand),
}

impl PropertyName {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Property::from_name(name).map_or_else(
            || Shorthand::from_name(name).map(PropertyName::Shorthand),
            |p| Some(Self::Longhand(p)),
        )
    }
}

macro_rules! impl_from {
    ($($variant:ident($ty:ty)),* $(,)?) => {
        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Value::$variant(v)
                }
            }
        )*
    };
}

macro_rules! impl_accessors {
    ($($name:ident -> $variant:ident($ty:ty)),* $(,)?) => {
        impl Value {
            $(
                pub const fn $name(&self) -> Option<&$ty> {
                    match self {
                        Value::$variant(v) => Some(v),
                        _ => None,
                    }
                }
            )*
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Display(Display),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    JustifyContent(JustifyContent),
    AlignItems(AlignItems),
    AlignSelf(AlignSelf),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    TextDecoration(TextDecoration),
    TextAlign(TextAlign),
    VerticalAlign(VerticalAlign),
    WhiteSpace(WhiteSpace),
    OverflowWrap(OverflowWrap),
    Overflow(Overflow),
    Visibility(Visibility),
    BorderStyle(BorderStyle),

    Length(Length),
    Dimension(Dimension),

    Color(Color),

    Number(f32),
    Integer(i16),

    Inherit,
    Initial,

    Unresolved(UnresolvedValue),
    Custom(CustomValue),
}

impl_from! {
    Display(Display),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    JustifyContent(JustifyContent),
    AlignItems(AlignItems),
    AlignSelf(AlignSelf),
    FontWeight(FontWeight),
    FontStyle(FontStyle),
    TextDecoration(TextDecoration),
    TextAlign(TextAlign),
    VerticalAlign(VerticalAlign),
    WhiteSpace(WhiteSpace),
    OverflowWrap(OverflowWrap),
    Overflow(Overflow),
    Visibility(Visibility),
    BorderStyle(BorderStyle),
    Length(Length),
    Dimension(Dimension),
    Color(Color),
}

impl_accessors! {
    as_display -> Display(Display),
    as_flex_direction -> FlexDirection(FlexDirection),
    as_flex_wrap -> FlexWrap(FlexWrap),
    as_justify_content -> JustifyContent(JustifyContent),
    as_align_items -> AlignItems(AlignItems),
    as_align_self -> AlignSelf(AlignSelf),
    as_font_weight -> FontWeight(FontWeight),
    as_font_style -> FontStyle(FontStyle),
    as_text_decoration -> TextDecoration(TextDecoration),
    as_text_align -> TextAlign(TextAlign),
    as_vertical_align -> VerticalAlign(VerticalAlign),
    as_white_space -> WhiteSpace(WhiteSpace),
    as_overflow_wrap -> OverflowWrap(OverflowWrap),
    as_overflow -> Overflow(Overflow),
    as_visibility -> Visibility(Visibility),
    as_border_style -> BorderStyle(BorderStyle),
    as_length -> Length(Length),
    as_dimension -> Dimension(Dimension),
    as_color -> Color(Color),
}

impl Value {
    #[must_use]
    pub const fn cells(cells: i16) -> Self {
        Self::Length(Length::Cells(cells))
    }

    #[must_use]
    pub const fn percent(percent: f32) -> Self {
        Self::Length(Length::Percent(percent))
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Dimension(Dimension::Auto)
    }

    #[must_use]
    pub const fn as_number(&self) -> Option<f32> {
        match self {
            Self::Number(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_integer(&self) -> Option<i16> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved(_))
    }

    #[must_use]
    pub const fn as_unresolved(&self) -> Option<&UnresolvedValue> {
        match self {
            Self::Unresolved(u) => Some(u),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_unresolved_css(&self) -> Option<&str> {
        self.as_unresolved().map(|u| u.css.as_str())
    }

    #[must_use]
    pub const fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    #[must_use]
    pub const fn as_custom(&self) -> Option<&CustomValue> {
        match self {
            Self::Custom(c) => Some(c),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_inherit(&self) -> bool {
        matches!(self, Self::Inherit)
    }

    #[must_use]
    pub const fn is_initial(&self) -> bool {
        matches!(self, Self::Initial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_from_name() {
        assert_eq!(Property::from_name("display"), Some(Property::Display));
        assert_eq!(Property::from_name("margin-top"), Some(Property::MarginTop));
        assert_eq!(Property::from_name("margin"), None); // shorthand
    }

    #[test]
    fn shorthand_from_name() {
        assert_eq!(Shorthand::from_name("margin"), Some(Shorthand::Margin));
        assert_eq!(Shorthand::from_name("margin-top"), None); // longhand
    }

    #[test]
    fn property_name_parses_both() {
        assert!(matches!(
            PropertyName::from_name("margin"),
            Some(PropertyName::Shorthand(Shorthand::Margin))
        ));
        assert!(matches!(
            PropertyName::from_name("margin-top"),
            Some(PropertyName::Longhand(Property::MarginTop))
        ));
        assert!(PropertyName::from_name("nonsense").is_none());
    }

    #[test]
    fn shorthand_longhands() {
        let longhands = Shorthand::Margin.longhands();
        assert_eq!(longhands.len(), 4);
        assert!(longhands.contains(&Property::MarginTop));
        assert!(longhands.contains(&Property::MarginLeft));
    }

    #[test]
    fn property_inherited() {
        assert!(Property::Color.inherited());
        assert!(Property::FontWeight.inherited());
        assert!(!Property::Display.inherited());
        assert!(!Property::MarginTop.inherited());
    }

    #[test]
    fn convenience_constructors() {
        assert_eq!(Value::cells(10), Value::Length(Length::Cells(10)));
        assert_eq!(Value::auto(), Value::Dimension(Dimension::Auto));
    }

    #[test]
    fn from_impls() {
        let v: Value = Display::Flex.into();
        assert_eq!(v.as_display(), Some(&Display::Flex));

        let v: Value = Color::RED.into();
        assert_eq!(v.as_color(), Some(&Color::RED));
    }

    #[test]
    fn global_keywords() {
        assert!(Value::Inherit.is_inherit());
        assert!(!Value::Inherit.is_initial());
        assert!(Value::Initial.is_initial());
    }
}
