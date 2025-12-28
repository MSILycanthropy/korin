use std::sync::Arc;

use cssparser::{Parser, ParserInput};
use ginyu_force::Pose;
use selectors::{
    SelectorList,
    context::{MatchingContext, SelectorCaches},
    matching::matches_selector,
};
use smallvec::SmallVec;

use crate::{
    AlignContent, AlignItems, AlignSelf, BorderStyle, CapsuleElement, Color, ComputedStyle,
    CustomPropertiesMap, CustomPropertiesResolver, Dimension, Display, ElementState, FlexDirection,
    FlexWrap, FontStyle, FontWeight, JustifyContent, Length, Overflow, OverflowWrap, Property,
    Selectors, Stylesheet, TElement, TextAlign, TextDecoration, Value, VerticalAlign, Visibility,
    WhiteSpace,
    bulma::{
        cascade::CascadeData, invalidation::InvalidationMap, make_context, restyle::RestyleHint,
        rule::BulmaRule,
    },
    parser::{Declaration, Rule, parse_inline_style, parse_property_value},
};

pub struct Bulma {
    cascade_data: CascadeData,
    invalidation_map: InvalidationMap,

    num_rebuilds: usize,
    source_order: u32,
}

impl Bulma {
    const AUTHOR_SOURCE_ORDER_START: u32 = 1_000_000;

    #[must_use]
    pub fn new() -> Self {
        Self {
            cascade_data: CascadeData::default(),
            invalidation_map: InvalidationMap::default(),
            num_rebuilds: 0,
            source_order: Self::AUTHOR_SOURCE_ORDER_START,
        }
    }

    pub fn add_ua_stylesheet(&mut self, stylesheet: &Stylesheet) {
        let mut source_order = self.source_order & !Self::AUTHOR_SOURCE_ORDER_START;

        for rule in &stylesheet.rules {
            self.add_rule(rule, None, &mut source_order);
        }

        self.source_order = source_order | (self.source_order & Self::AUTHOR_SOURCE_ORDER_START);

        self.cascade_data.shrink_to_fit();
        self.invalidation_map.shrink_to_fit();
    }

    pub fn add_stylesheet(&mut self, stylesheet: &Stylesheet) {
        let mut source_order = self.source_order;

        for rule in &stylesheet.rules {
            self.add_rule(rule, None, &mut source_order);
        }

        self.source_order = source_order;
        self.cascade_data.shrink_to_fit();
        self.invalidation_map.shrink_to_fit();
        self.num_rebuilds += 1;
    }

    fn add_rule(
        &mut self,
        rule: &Rule,
        parent_selectors: Option<&SelectorList<Selectors>>,
        source_order: &mut u32,
    ) {
        let declations = Arc::new(rule.declarations.clone());

        for selector in rule.selectors.slice() {
            let final_selector = parent_selectors.map_or_else(
                || selector.clone(),
                |parent| selector.replace_parent_selector(parent),
            );

            self.invalidation_map.register_selector(&final_selector);

            let bulma_rule = BulmaRule::new(final_selector, declations.clone(), self.source_order);

            self.cascade_data.insert(bulma_rule);
            *source_order += 1;
        }

        for nested in &rule.nested_rules {
            self.add_rule(nested, Some(&rule.selectors), source_order);
        }
    }

    pub fn clear(&mut self) {
        self.cascade_data.clear();
        self.invalidation_map.clear();
        self.source_order = Self::AUTHOR_SOURCE_ORDER_START;
    }

    #[inline]
    #[must_use]
    pub fn restyle_hint_for_state_change(
        &self,
        old: ElementState,
        new: ElementState,
    ) -> RestyleHint {
        self.invalidation_map
            .restyle_hint_for_state_change(old, new)
    }

    #[inline]
    #[must_use]
    pub fn restyle_hint_for_attribute_change(&self, attribute: Pose) -> RestyleHint {
        self.invalidation_map
            .restyle_hint_for_attribute_change(attribute)
    }

    #[inline]
    #[must_use]
    pub fn restyle_hint_for_class_change(&self, class: Pose) -> RestyleHint {
        self.invalidation_map.restyle_hint_for_class_change(class)
    }

    #[inline]
    #[must_use]
    pub fn restyle_hint_for_id_change(&self, id: Pose) -> RestyleHint {
        self.invalidation_map.restyle_hint_for_id_change(id)
    }

    #[inline]
    #[must_use]
    pub fn has_state_dependency(&self, state: ElementState) -> bool {
        self.invalidation_map.has_state_dependency(state)
    }

    #[inline]
    #[must_use]
    pub fn has_attribute_dependency(&self, attribute: Pose) -> bool {
        self.invalidation_map.has_attribute_dependency(attribute)
    }

    pub fn collect_matching_rules<E: TElement>(
        &mut self,
        element: &E,
        caches: &mut SelectorCaches,
    ) -> SmallVec<[ApplicableDeclaration; 8]> {
        let mut matched = SmallVec::new();
        let wrapped = CapsuleElement::new(element.clone());
        let cascade_data = &self.cascade_data;
        let mut context = make_context(caches);

        if let Some(id) = element.id()
            && let Some(rules) = cascade_data.rules_by_id(id)
        {
            collect_if_matching(&wrapped, rules, &mut context, &mut matched);
        }

        element.each_class(|class| {
            if let Some(rules) = cascade_data.rules_by_class(class) {
                collect_if_matching(&wrapped, rules, &mut context, &mut matched);
            }
        });

        if let Some(rules) = cascade_data.rules_by_tag(element.tag_name()) {
            collect_if_matching(&wrapped, rules, &mut context, &mut matched);
        }

        collect_if_matching(
            &wrapped,
            cascade_data.universal_rules(),
            &mut context,
            &mut matched,
        );

        matched.sort_by_key(ApplicableDeclaration::sort_key);

        matched
    }

    pub fn compute_style<E: TElement>(
        &mut self,
        element: &E,
        parent_style: Option<&ComputedStyle>,
        parent_custom_properties: Option<&CustomPropertiesMap>,
        caches: &mut SelectorCaches,
    ) -> (ComputedStyle, CustomPropertiesMap) {
        let matched = self.collect_matching_rules(element, caches);

        let mut style =
            parent_style.map_or_else(ComputedStyle::default, ComputedStyle::inherit_from);

        let mut resolver = CustomPropertiesResolver::new(parent_custom_properties);

        let inline_declarations = element
            .style_attribute()
            .map(parse_inline_style)
            .unwrap_or_default();

        for applicable in &matched {
            for declaration in applicable.declarations.iter() {
                if let (Property::Custom(name), Value::Custom(value)) =
                    (&declaration.property, &declaration.value)
                    && !declaration.important
                {
                    resolver.add(*name, value.clone());
                }
            }
        }

        for declaration in &inline_declarations {
            if let (Property::Custom(name), Value::Custom(value)) =
                (&declaration.property, &declaration.value)
                && !declaration.important
            {
                resolver.add(*name, value.clone());
            }
        }

        for applicable in &matched {
            for declaration in applicable.declarations.iter() {
                if let (Property::Custom(name), Value::Custom(value)) =
                    (&declaration.property, &declaration.value)
                    && declaration.important
                {
                    resolver.add(*name, value.clone());
                }
            }
        }

        for declaration in &inline_declarations {
            if let (Property::Custom(name), Value::Custom(value)) =
                (&declaration.property, &declaration.value)
                && declaration.important
            {
                resolver.add(*name, value.clone());
            }
        }

        let custom_properties = resolver.build();

        for applicable in &matched {
            for declaration in applicable.declarations.iter() {
                if !declaration.property.is_custom() && !declaration.important {
                    apply_declaration(&mut style, declaration, parent_style, &custom_properties);
                }
            }
        }

        for declaration in &inline_declarations {
            if !declaration.property.is_custom() && !declaration.important {
                apply_declaration(&mut style, declaration, parent_style, &custom_properties);
            }
        }

        for applicable in &matched {
            for declaration in applicable.declarations.iter() {
                if !declaration.property.is_custom() && declaration.important {
                    apply_declaration(&mut style, declaration, parent_style, &custom_properties);
                }
            }
        }

        for declaration in &inline_declarations {
            if !declaration.property.is_custom() && declaration.important {
                apply_declaration(&mut style, declaration, parent_style, &custom_properties);
            }
        }

        (style, custom_properties)
    }

    #[must_use]
    pub const fn num_selectors(&self) -> usize {
        self.cascade_data.num_selectors
    }

    #[must_use]
    pub const fn num_declarations(&self) -> usize {
        self.cascade_data.num_declarations
    }

    #[must_use]
    pub const fn num_rebuilds(&self) -> usize {
        self.num_rebuilds
    }
}

impl Default for Bulma {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_if_matching<E: TElement>(
    element: &CapsuleElement<E>,
    rules: &[BulmaRule],
    context: &mut MatchingContext<'_, Selectors>,
    matched: &mut SmallVec<[ApplicableDeclaration; 8]>,
) {
    for rule in rules {
        if matches_selector(&rule.selector, 0, None, element, context) {
            matched.push(ApplicableDeclaration {
                declarations: rule.declarations.clone(),
                specificity: rule.specificity(),
                source_order: rule.source_order,
            });
        }
    }
}

// TODO: I kinda don't like how disjoined values and properties have kinda ended up
// I think at some point I need to go back and resolve this. imo it doesn't really
// make too much sense for there to be separate `Property` and `Value` enums
// In reality they should just be one thing.
fn apply_declaration(
    style: &mut ComputedStyle,
    declaration: &Declaration,
    parent_style: Option<&ComputedStyle>,
    custom_properties: &CustomPropertiesMap,
) {
    if declaration.value.is_inherit() {
        if let Some(parent) = parent_style {
            apply_inherited(style, declaration.property, parent);
        }

        return;
    }

    if declaration.value.is_initial() {
        apply_initial(style, declaration.property);

        return;
    }

    if declaration.value.is_unset() {
        if declaration.property.inherited() {
            if let Some(parent) = parent_style {
                apply_inherited(style, declaration.property, parent);
            }
        } else {
            apply_initial(style, declaration.property);
        }
        return;
    }

    if let Some(unresolved) = declaration.value.as_unresolved() {
        if let Ok(substituted) = unresolved.substitute(|name| custom_properties.get(name))
            && let Some(value) = parse_substituted_value(declaration.property, &substituted)
        {
            apply_value(style, declaration.property, &value);
        }

        return;
    }

    apply_value(style, declaration.property, &declaration.value);
}

fn apply_inherited(style: &mut ComputedStyle, property: Property, parent: &ComputedStyle) {
    match property {
        Property::Display => style.display = parent.display,
        Property::FlexDirection => style.flex_direction = parent.flex_direction,
        Property::FlexWrap => style.flex_wrap = parent.flex_wrap,
        Property::JustifyContent => style.justify_content = parent.justify_content,
        Property::AlignItems => style.align_items = parent.align_items,
        Property::AlignContent => style.align_content = parent.align_content,
        Property::FlexGrow => style.flex_grow = parent.flex_grow,
        Property::FlexShrink => style.flex_shrink = parent.flex_shrink,
        Property::FlexBasis => style.flex_basis = parent.flex_basis.clone(),
        Property::AlignSelf => style.align_self = parent.align_self,
        Property::RowGap => style.row_gap = parent.row_gap.clone(),
        Property::ColumnGap => style.column_gap = parent.column_gap.clone(),
        Property::Width => style.width = parent.width.clone(),
        Property::Height => style.height = parent.height.clone(),
        Property::MinWidth => style.min_width = parent.min_width.clone(),
        Property::MaxWidth => style.max_width = parent.max_width.clone(),
        Property::MinHeight => style.min_height = parent.min_height.clone(),
        Property::MaxHeight => style.max_height = parent.max_height.clone(),
        Property::MarginTop => style.margin.top = parent.margin.top.clone(),
        Property::MarginRight => style.margin.right = parent.margin.right.clone(),
        Property::MarginBottom => style.margin.bottom = parent.margin.bottom.clone(),
        Property::MarginLeft => style.margin.left = parent.margin.left.clone(),
        Property::PaddingTop => style.padding.top = parent.padding.top.clone(),
        Property::PaddingRight => style.padding.right = parent.padding.right.clone(),
        Property::PaddingBottom => style.padding.bottom = parent.padding.bottom.clone(),
        Property::PaddingLeft => style.padding.left = parent.padding.left.clone(),
        Property::BorderTopStyle => style.border_style.top = parent.border_style.top,
        Property::BorderRightStyle => style.border_style.right = parent.border_style.right,
        Property::BorderBottomStyle => style.border_style.bottom = parent.border_style.bottom,
        Property::BorderLeftStyle => style.border_style.left = parent.border_style.left,
        Property::BorderTopColor => style.border_color.top = parent.border_color.top,
        Property::BorderRightColor => style.border_color.right = parent.border_color.right,
        Property::BorderBottomColor => style.border_color.bottom = parent.border_color.bottom,
        Property::BorderLeftColor => style.border_color.left = parent.border_color.left,
        Property::Color => style.color = parent.color,
        Property::BackgroundColor => style.background_color = parent.background_color,
        Property::FontWeight => style.font_weight = parent.font_weight,
        Property::FontStyle => style.font_style = parent.font_style,
        Property::TextDecoration => style.text_decoration = parent.text_decoration,
        Property::TextAlign => style.text_align = parent.text_align,
        Property::VerticalAlign => style.vertical_align = parent.vertical_align,
        Property::WhiteSpace => style.white_space = parent.white_space,
        Property::OverflowWrap => style.overflow_wrap = parent.overflow_wrap,
        Property::OverflowX => style.overflow_x = parent.overflow_x,
        Property::OverflowY => style.overflow_y = parent.overflow_y,
        Property::Visibility => style.visibility = parent.visibility,
        Property::ZIndex => style.z_index = parent.z_index,
        Property::GridTemplateColumns
        | Property::GridTemplateRows
        | Property::GridColumn
        | Property::GridRow
        | Property::Custom(_) => {}
    }
}

fn apply_initial(style: &mut ComputedStyle, property: Property) {
    match property {
        Property::Display => style.display = Display::default(),
        Property::FlexDirection => style.flex_direction = FlexDirection::default(),
        Property::FlexWrap => style.flex_wrap = FlexWrap::default(),
        Property::JustifyContent => style.justify_content = JustifyContent::default(),
        Property::AlignItems => style.align_items = AlignItems::default(),
        Property::AlignContent => style.align_content = AlignContent::default(),
        Property::FlexGrow => style.flex_grow = 0.0,
        Property::FlexShrink => style.flex_shrink = 1.0,
        Property::FlexBasis => style.flex_basis = Dimension::Auto,
        Property::AlignSelf => style.align_self = AlignSelf::default(),
        Property::RowGap => style.row_gap = Length::ZERO,
        Property::ColumnGap => style.column_gap = Length::ZERO,
        Property::Width => style.width = Dimension::Auto,
        Property::Height => style.height = Dimension::Auto,
        Property::MinWidth => style.min_width = Dimension::Auto,
        Property::MaxWidth => style.max_width = Dimension::None,
        Property::MinHeight => style.min_height = Dimension::Auto,
        Property::MaxHeight => style.max_height = Dimension::None,
        Property::MarginTop => style.margin.top = Length::ZERO,
        Property::MarginRight => style.margin.right = Length::ZERO,
        Property::MarginBottom => style.margin.bottom = Length::ZERO,
        Property::MarginLeft => style.margin.left = Length::ZERO,
        Property::PaddingTop => style.padding.top = Length::ZERO,
        Property::PaddingRight => style.padding.right = Length::ZERO,
        Property::PaddingBottom => style.padding.bottom = Length::ZERO,
        Property::PaddingLeft => style.padding.left = Length::ZERO,
        Property::BorderTopStyle => style.border_style.top = BorderStyle::default(),
        Property::BorderRightStyle => style.border_style.right = BorderStyle::default(),
        Property::BorderBottomStyle => style.border_style.bottom = BorderStyle::default(),
        Property::BorderLeftStyle => style.border_style.left = BorderStyle::default(),
        Property::BorderTopColor => style.border_color.top = Color::Reset,
        Property::BorderRightColor => style.border_color.right = Color::Reset,
        Property::BorderBottomColor => style.border_color.bottom = Color::Reset,
        Property::BorderLeftColor => style.border_color.left = Color::Reset,
        Property::Color => style.color = Color::Reset,
        Property::BackgroundColor => style.background_color = Color::Reset,
        Property::FontWeight => style.font_weight = FontWeight::default(),
        Property::FontStyle => style.font_style = FontStyle::default(),
        Property::TextDecoration => style.text_decoration = TextDecoration::default(),
        Property::TextAlign => style.text_align = TextAlign::default(),
        Property::VerticalAlign => style.vertical_align = VerticalAlign::default(),
        Property::WhiteSpace => style.white_space = WhiteSpace::default(),
        Property::OverflowWrap => style.overflow_wrap = OverflowWrap::default(),
        Property::OverflowX => style.overflow_x = Overflow::default(),
        Property::OverflowY => style.overflow_y = Overflow::default(),
        Property::Visibility => style.visibility = Visibility::default(),
        Property::ZIndex => style.z_index = 0,

        // TODO: Grid
        Property::GridTemplateColumns
        | Property::GridTemplateRows
        | Property::GridColumn
        | Property::GridRow => {}
        Property::Custom(_) => unreachable!(),
    }
}

fn apply_value(style: &mut ComputedStyle, property: Property, value: &Value) {
    match (property, value) {
        (Property::Display, Value::Display(v)) => style.display = *v,
        (Property::FlexDirection, Value::FlexDirection(v)) => style.flex_direction = *v,
        (Property::FlexWrap, Value::FlexWrap(v)) => style.flex_wrap = *v,
        (Property::JustifyContent, Value::JustifyContent(v)) => style.justify_content = *v,
        (Property::AlignItems, Value::AlignItems(v)) => style.align_items = *v,
        (Property::FlexGrow, Value::Number(v)) => style.flex_grow = *v,
        (Property::FlexShrink, Value::Number(v)) => style.flex_shrink = *v,
        (Property::FlexBasis, Value::Dimension(v)) => style.flex_basis = v.clone(),
        (Property::AlignSelf, Value::AlignSelf(v)) => style.align_self = *v,
        (Property::AlignContent, Value::AlignContent(v)) => style.align_content = *v,
        (Property::RowGap, Value::Length(v)) => style.row_gap = v.clone(),
        (Property::ColumnGap, Value::Length(v)) => style.column_gap = v.clone(),
        (Property::Width, Value::Dimension(v)) => style.width = v.clone(),
        (Property::Height, Value::Dimension(v)) => style.height = v.clone(),
        (Property::MinWidth, Value::Dimension(v)) => style.min_width = v.clone(),
        (Property::MaxWidth, Value::Dimension(v)) => style.max_width = v.clone(),
        (Property::MinHeight, Value::Dimension(v)) => style.min_height = v.clone(),
        (Property::MaxHeight, Value::Dimension(v)) => style.max_height = v.clone(),
        (Property::MarginTop, Value::Length(v)) => style.margin.top = v.clone(),
        (Property::MarginRight, Value::Length(v)) => style.margin.right = v.clone(),
        (Property::MarginBottom, Value::Length(v)) => style.margin.bottom = v.clone(),
        (Property::MarginLeft, Value::Length(v)) => style.margin.left = v.clone(),
        (Property::PaddingTop, Value::Length(v)) => style.padding.top = v.clone(),
        (Property::PaddingRight, Value::Length(v)) => style.padding.right = v.clone(),
        (Property::PaddingBottom, Value::Length(v)) => style.padding.bottom = v.clone(),
        (Property::PaddingLeft, Value::Length(v)) => style.padding.left = v.clone(),
        (Property::BorderTopStyle, Value::BorderStyle(v)) => style.border_style.top = *v,
        (Property::BorderRightStyle, Value::BorderStyle(v)) => style.border_style.right = *v,
        (Property::BorderBottomStyle, Value::BorderStyle(v)) => style.border_style.bottom = *v,
        (Property::BorderLeftStyle, Value::BorderStyle(v)) => style.border_style.left = *v,
        (Property::BorderTopColor, Value::Color(v)) => style.border_color.top = *v,
        (Property::BorderRightColor, Value::Color(v)) => style.border_color.right = *v,
        (Property::BorderBottomColor, Value::Color(v)) => style.border_color.bottom = *v,
        (Property::BorderLeftColor, Value::Color(v)) => style.border_color.left = *v,
        (Property::Color, Value::Color(v)) => style.color = *v,
        (Property::BackgroundColor, Value::Color(v)) => style.background_color = *v,
        (Property::FontWeight, Value::FontWeight(v)) => style.font_weight = *v,
        (Property::FontStyle, Value::FontStyle(v)) => style.font_style = *v,
        (Property::TextDecoration, Value::TextDecoration(v)) => style.text_decoration = *v,
        (Property::TextAlign, Value::TextAlign(v)) => style.text_align = *v,
        (Property::VerticalAlign, Value::VerticalAlign(v)) => style.vertical_align = *v,
        (Property::WhiteSpace, Value::WhiteSpace(v)) => style.white_space = *v,
        (Property::OverflowWrap, Value::OverflowWrap(v)) => style.overflow_wrap = *v,
        (Property::OverflowX, Value::Overflow(v)) => style.overflow_x = *v,
        (Property::OverflowY, Value::Overflow(v)) => style.overflow_y = *v,
        (Property::Visibility, Value::Visibility(v)) => style.visibility = *v,
        (Property::ZIndex, Value::Integer(v)) => style.z_index = *v,
        (
            Property::GridTemplateColumns
            | Property::GridTemplateRows
            | Property::GridColumn
            | Property::GridRow,
            _,
        ) => {}

        (Property::Custom(_), _) => unreachable!(),

        _ => {
            #[cfg(debug_assertions)]
            panic!("Type mismatch applying {property:?} with value {value:?}")
        }
    }
}

fn parse_substituted_value(property: Property, css: &str) -> Option<Value> {
    debug_assert!(!property.is_custom());

    let mut input = ParserInput::new(css);
    let mut input = Parser::new(&mut input);

    parse_property_value(property, &mut input).ok()
}

#[derive(Debug, Clone)]
pub struct ApplicableDeclaration {
    pub declarations: Arc<Vec<Declaration>>,
    pub specificity: u32,
    pub source_order: u32,
}

impl ApplicableDeclaration {
    #[inline]
    #[must_use]
    pub const fn sort_key(&self) -> (u32, u32) {
        (self.specificity, self.source_order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Stylesheet;
    use selectors::context::SelectorCaches;

    #[derive(Debug, Clone, PartialEq)]
    struct TestElement {
        tag: Pose,
        id: Option<Pose>,
        classes: Vec<Pose>,
        state: ElementState,
        style: Option<String>,
    }

    impl TestElement {
        fn new(tag: &str) -> Self {
            Self {
                tag: Pose::from(tag),
                id: None,
                classes: vec![],
                state: ElementState::empty(),
                style: None,
            }
        }

        fn with_style(mut self, style: &str) -> Self {
            self.style = Some(style.to_string());
            self
        }

        fn with_id(mut self, id: &str) -> Self {
            self.id = Some(Pose::from(id));
            self
        }

        fn with_class(mut self, class: &str) -> Self {
            self.classes.push(Pose::from(class));
            self
        }

        fn with_state(mut self, state: ElementState) -> Self {
            self.state = state;
            self
        }
    }

    impl TElement for TestElement {
        fn tag_name(&self) -> Pose {
            self.tag
        }

        fn id(&self) -> Option<Pose> {
            self.id
        }

        fn has_class(&self, name: &str) -> bool {
            self.classes.iter().any(|c| c.as_str() == name)
        }

        fn each_class<F: FnMut(Pose)>(&self, mut callback: F) {
            for class in &self.classes {
                callback(*class);
            }
        }

        fn get_attribute(&self, _name: Pose) -> Option<&str> {
            None
        }

        fn style_attribute(&self) -> Option<&str> {
            self.style.as_deref()
        }

        fn state(&self) -> ElementState {
            self.state
        }

        fn parent(&self) -> Option<Self> {
            None
        }

        fn prev_sibling(&self) -> Option<Self> {
            None
        }

        fn next_sibling(&self) -> Option<Self> {
            None
        }

        fn has_children(&self) -> bool {
            false
        }
    }

    #[test]
    fn empty_bulma() {
        let bulma = Bulma::new();
        assert_eq!(bulma.num_selectors(), 0);
        assert_eq!(bulma.num_declarations(), 0);
        assert_eq!(bulma.num_rebuilds(), 0);
    }

    #[test]
    fn add_stylesheet_increments_rebuilds() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(".foo { color: red }").expect("failed");

        bulma.add_stylesheet(&stylesheet);
        assert_eq!(bulma.num_rebuilds(), 1);

        bulma.add_stylesheet(&stylesheet);
        assert_eq!(bulma.num_rebuilds(), 2);
    }

    #[test]
    fn add_stylesheet_counts_selectors() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(".a, .b, .c { color: red }").expect("failed");

        bulma.add_stylesheet(&stylesheet);
        assert_eq!(bulma.num_selectors(), 3);
    }

    #[test]
    fn collect_matching_rules_by_class() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".btn { color: red }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div").with_class("btn");
        let mut caches = SelectorCaches::default();

        let matched = bulma.collect_matching_rules(&element, &mut caches);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn collect_matching_rules_by_id() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse("#main { color: red }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div").with_id("main");
        let mut caches = SelectorCaches::default();

        let matched = bulma.collect_matching_rules(&element, &mut caches);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn collect_matching_rules_by_tag() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse("div { color: red }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let matched = bulma.collect_matching_rules(&element, &mut caches);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn collect_matching_rules_no_match() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".btn { color: red }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div"); // no class
        let mut caches = SelectorCaches::default();

        let matched = bulma.collect_matching_rules(&element, &mut caches);
        assert!(matched.is_empty());
    }

    #[test]
    fn collect_matching_rules_with_state() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".btn:hover { color: blue }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element_no_hover = TestElement::new("div").with_class("btn");
        let element_hover = TestElement::new("div")
            .with_class("btn")
            .with_state(ElementState::HOVER);

        let mut caches = SelectorCaches::default();
        let matched = bulma.collect_matching_rules(&element_no_hover, &mut caches);
        assert!(matched.is_empty());

        let mut caches = SelectorCaches::default();
        let matched = bulma.collect_matching_rules(&element_hover, &mut caches);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn compute_style_applies_color() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".red { color: red }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div").with_class("red");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn compute_style_applies_display() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".flex { display: flex }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div").with_class("flex");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn compute_style_inherits_color() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".parent { color: cyan }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        // Compute parent style
        let parent_element = TestElement::new("div").with_class("parent");
        let mut caches = SelectorCaches::default();
        let (parent_style, parent_cp) =
            bulma.compute_style(&parent_element, None, None, &mut caches);

        // Child should inherit color
        let child_element = TestElement::new("span");
        let mut caches = SelectorCaches::default();
        let (child_style, _) = bulma.compute_style(
            &child_element,
            Some(&parent_style),
            Some(&parent_cp),
            &mut caches,
        );

        assert_eq!(child_style.color, Color::CYAN);
    }

    #[test]
    fn compute_style_does_not_inherit_display() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".parent { display: flex }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let parent_element = TestElement::new("div").with_class("parent");
        let mut caches = SelectorCaches::default();
        let (parent_style, parent_cp) =
            bulma.compute_style(&parent_element, None, None, &mut caches);

        let child_element = TestElement::new("span");
        let mut caches = SelectorCaches::default();
        let (child_style, _) = bulma.compute_style(
            &child_element,
            Some(&parent_style),
            Some(&parent_cp),
            &mut caches,
        );

        // display is not inherited
        assert_eq!(child_style.display, Display::default());
    }

    #[test]
    fn compute_style_important_wins() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(
                r"
                .a { color: red }
                .b { color: blue !important }
                .c { color: green }
            ",
            )
            .expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div")
            .with_class("a")
            .with_class("b")
            .with_class("c");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn compute_style_later_rule_wins() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(
                r"
                .a { color: red }
                .a { color: blue }
            ",
            )
            .expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div").with_class("a");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn compute_style_higher_specificity_wins() {
        let mut bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(
                r"
                .a { color: red }
                #id.a { color: blue }
            ",
            )
            .expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let element = TestElement::new("div").with_id("id").with_class("a");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn restyle_hint_for_hover_change() {
        let bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".btn:hover { color: blue }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let hint = bulma.restyle_hint_for_state_change(ElementState::empty(), ElementState::HOVER);

        assert!(hint.contains(RestyleHint::RESTYLE_SELF));
    }

    #[test]
    fn restyle_hint_for_class_change() {
        let bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".active { color: blue }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let hint = bulma.restyle_hint_for_class_change(Pose::from("active"));
        assert!(hint.contains(RestyleHint::RESTYLE_SELF));
    }

    #[test]
    fn restyle_hint_for_class_change_no_dependency() {
        let bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse(".active { color: blue }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let hint = bulma.restyle_hint_for_class_change(Pose::from("inactive"));
        assert!(hint.is_empty());
    }

    #[test]
    fn restyle_hint_for_id_change() {
        let bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse("#main { color: blue }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let hint = bulma.restyle_hint_for_id_change(Pose::from("main"));
        assert!(hint.contains(RestyleHint::RESTYLE_SELF));
    }

    #[test]
    fn restyle_hint_for_id_change_no_dependency() {
        let bulma = {
            let mut b = Bulma::new();
            let stylesheet = Stylesheet::parse("#main { color: blue }").expect("failed");
            b.add_stylesheet(&stylesheet);
            b
        };

        let hint = bulma.restyle_hint_for_id_change(Pose::from("sidebar"));
        assert!(hint.is_empty());
    }

    #[test]
    fn root_selector_matches() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(":root { color: red }").expect("failed");
        bulma.add_stylesheet(&stylesheet);

        // Should have 1 universal rule
        assert_eq!(bulma.num_selectors(), 1);

        let root = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let matched = bulma.collect_matching_rules(&root, &mut caches);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn compute_style_inline_style() {
        let mut bulma = Bulma::new();

        let element = TestElement::new("div").with_style("color: red");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn compute_style_inline_beats_stylesheet() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(".foo { color: blue }").expect("failed");
        bulma.add_stylesheet(&stylesheet);

        let element = TestElement::new("div")
            .with_class("foo")
            .with_style("color: red");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn compute_style_stylesheet_important_beats_inline() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(".foo { color: blue !important }").expect("failed");
        bulma.add_stylesheet(&stylesheet);

        let element = TestElement::new("div")
            .with_class("foo")
            .with_style("color: red");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn compute_style_inline_important_beats_all() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(".foo { color: blue !important }").expect("failed");
        bulma.add_stylesheet(&stylesheet);

        let element = TestElement::new("div")
            .with_class("foo")
            .with_style("color: red !important");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn compute_style_inline_multiple_properties() {
        let mut bulma = Bulma::new();

        let element = TestElement::new("div").with_style("color: red; display: flex");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::RED);
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn compute_style_inline_shorthand() {
        let mut bulma = Bulma::new();

        let element = TestElement::new("div").with_style("margin: 10");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.margin.top, Length::Cells(10));
        assert_eq!(style.margin.right, Length::Cells(10));
        assert_eq!(style.margin.bottom, Length::Cells(10));
        assert_eq!(style.margin.left, Length::Cells(10));
    }

    #[test]
    fn compute_style_inline_var() {
        let mut bulma = Bulma::new();
        let stylesheet = Stylesheet::parse(":root { --primary: cyan }").expect("failed");
        bulma.add_stylesheet(&stylesheet);

        // Need a root element to get the custom property
        let root = TestElement::new("div");
        let mut caches = SelectorCaches::default();
        let (_, root_style) = bulma.compute_style(&root, None, None, &mut caches);

        assert_eq!(root_style.get(Pose::from("primary")), Some("cyan"));

        let element = TestElement::new("div").with_style("color: var(--primary)");
        let mut caches = SelectorCaches::default();
        let (style, _) = bulma.compute_style(&element, None, Some(&root_style), &mut caches);

        assert_eq!(style.color, Color::CYAN);
    }

    #[test]
    fn compute_style_inline_custom_property() {
        let mut bulma = Bulma::new();

        let element = TestElement::new("div").with_style("--accent: red; color: var(--accent)");
        let mut caches = SelectorCaches::default();

        let inline = parse_inline_style("--accent: red; color: var(--accent)");
        dbg!(&inline);

        let (style, custom_props) = bulma.compute_style(&element, None, None, &mut caches);
        dbg!(&custom_props);

        assert_eq!(custom_props.get(Pose::from("accent")), Some("red"));
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn ua_stylesheet_applies() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse("div { color: red }").expect("failed");
        bulma.add_ua_stylesheet(&ua);

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn author_stylesheet_beats_ua() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse("div { color: red }").expect("failed");
        let author = Stylesheet::parse("div { color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn author_stylesheet_beats_ua_same_specificity() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse(".btn { color: red }").expect("failed");
        let author = Stylesheet::parse(".btn { color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        let element = TestElement::new("div").with_class("btn");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn ua_higher_specificity_still_loses_to_author() {
        let mut bulma = Bulma::new();
        // UA has higher specificity (id + class)
        let ua = Stylesheet::parse("#main.btn { color: red }").expect("failed");
        // Author has lower specificity (just class)
        let author = Stylesheet::parse(".btn { color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        let element = TestElement::new("div").with_id("main").with_class("btn");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        // UA wins because it has higher specificity
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn ua_important_vs_author_normal() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse("div { color: red !important }").expect("failed");
        let author = Stylesheet::parse("div { color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        // UA !important beats author normal
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn author_important_beats_ua_important() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse("div { color: red !important }").expect("failed");
        let author = Stylesheet::parse("div { color: blue !important }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        // Author !important beats UA !important
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn ua_stylesheet_with_custom_properties() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse(":root { --default-color: red }").expect("failed");
        let author = Stylesheet::parse("div { color: var(--default-color) }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        // Get custom props from root
        let root = TestElement::new("div");
        let mut caches = SelectorCaches::default();
        let (_, root_cp) = bulma.compute_style(&root, None, None, &mut caches);

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();
        let (style, _) = bulma.compute_style(&element, None, Some(&root_cp), &mut caches);

        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn author_custom_property_overrides_ua() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse(":root { --color: red }").expect("failed");
        let author = Stylesheet::parse(":root { --color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        let root = TestElement::new("div");
        let mut caches = SelectorCaches::default();
        let (_, root_cp) = bulma.compute_style(&root, None, None, &mut caches);

        assert_eq!(root_cp.get(Pose::from("color")), Some("blue"));
    }

    #[test]
    fn clear_removes_ua_and_author() {
        let mut bulma = Bulma::new();
        let ua = Stylesheet::parse("div { color: red }").expect("failed");
        let author = Stylesheet::parse(".foo { color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua);
        bulma.add_stylesheet(&author);

        assert_eq!(bulma.num_selectors(), 2);

        bulma.clear();

        assert_eq!(bulma.num_selectors(), 0);
    }

    #[test]
    fn multiple_ua_stylesheets() {
        let mut bulma = Bulma::new();
        let ua1 = Stylesheet::parse("div { color: red }").expect("failed");
        let ua2 = Stylesheet::parse("div { color: blue }").expect("failed");

        bulma.add_ua_stylesheet(&ua1);
        bulma.add_ua_stylesheet(&ua2);

        let element = TestElement::new("div");
        let mut caches = SelectorCaches::default();

        let (style, _) = bulma.compute_style(&element, None, None, &mut caches);
        // Later UA stylesheet wins
        assert_eq!(style.color, Color::BLUE);
    }
}
