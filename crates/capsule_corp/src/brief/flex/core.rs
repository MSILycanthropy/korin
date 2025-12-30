use crate::{AlignSelf, Edges, brief::box_model::ResolvedBox};

#[derive(Clone, Copy)]
pub struct FlexItem<NodeId: Clone + Copy> {
    pub node_id: NodeId,
    pub align_self: AlignSelf,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: u16,
    pub min_main_size: u16,
    pub max_main_size: Option<u16>,
    pub hypothetical_main_size: u16,
    pub margin: Edges<u16>,
    pub resolved_box: ResolvedBox,
    pub frozen: bool,
    pub main_size: u16,
    pub cross_size: u16,
    pub main_position: u16,
    pub cross_position: u16,
}

impl<NodeId: Copy> FlexItem<NodeId> {
    pub const fn outer_main_size(&self) -> u16 {
        self.main_size
            .saturating_add(self.margin.left)
            .saturating_add(self.margin.right)
    }

    pub const fn outer_cross_size(&self) -> u16 {
        self.cross_size
            .saturating_add(self.margin.top)
            .saturating_add(self.margin.bottom)
    }

    pub const fn outer_hypothetical_main_size(&self) -> u16 {
        self.hypothetical_main_size
            .saturating_add(self.margin.left)
            .saturating_add(self.margin.right)
    }
}

#[derive(Clone)]
pub struct FlexLine<NodeId: Clone + Copy> {
    pub items: Vec<FlexItem<NodeId>>,
    pub cross_size: u16,
    pub cross_position: u16,
}

impl<NodeId: Copy> FlexLine<NodeId> {
    pub const fn new() -> Self {
        Self {
            items: Vec::new(),
            cross_size: 0,
            cross_position: 0,
        }
    }
}

impl<NodeId: Copy> Default for FlexLine<NodeId> {
    fn default() -> Self {
        Self::new()
    }
}
