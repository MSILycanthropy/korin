pub struct FocusManager<Id>
where
    Id: Copy + Eq,
{
    order: Vec<Id>,
    index: usize,
}

pub struct FocusChange<Id>
where
    Id: Copy + Eq,
{
    prev: Option<Id>,
    next: Option<Id>,
}

impl<Id> FocusChange<Id>
where
    Id: Copy + Eq,
{
    const EMPTY: Self = Self {
        prev: None,
        next: None,
    };

    pub fn relevant(&self) -> bool {
        self.prev != self.next
    }

    pub const fn prev(&self) -> Option<Id> {
        self.prev
    }

    pub const fn next(&self) -> Option<Id> {
        self.next
    }
}

impl<Id: Copy + Eq> FocusManager<Id> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            order: Vec::new(),
            index: 0,
        }
    }

    #[must_use]
    pub fn focused(&self) -> Option<Id> {
        self.order.get(self.index).copied()
    }

    pub fn is_focused(&self, id: Id) -> bool {
        self.focused() == Some(id)
    }

    pub fn focus(&mut self, id: Id) -> bool {
        if let Some(position) = self.get_pos(id) {
            self.index = position;

            return true;
        }

        false
    }

    pub fn focus_next(&mut self) -> FocusChange<Id> {
        if self.order.is_empty() {
            return FocusChange::EMPTY;
        }

        let to = (self.index + 1) % self.len();

        self.change_focus(to)
    }

    pub fn focus_prev(&mut self) -> FocusChange<Id> {
        if self.order.is_empty() {
            return FocusChange::EMPTY;
        }

        let to = self.index.checked_sub(1).unwrap_or(self.order.len() - 1);

        self.change_focus(to)
    }

    pub fn set_order(&mut self, order: Vec<Id>) {
        let current = self.focused();

        self.order = order;

        if let Some(position) = current.and_then(|id| self.get_pos(id)) {
            self.index = position;
            return;
        }

        self.index = self.index.min(self.len().saturating_sub(1));
    }

    pub fn get_pos(&self, id: Id) -> Option<usize> {
        self.order.iter().position(|&n| n == id)
    }

    pub fn clear(&mut self) {
        self.order.clear();
        self.index = 0;
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.order.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    fn change_focus(&mut self, to: usize) -> FocusChange<Id> {
        let prev = self.focused();

        self.index = to;

        let next = self.focused();

        FocusChange { prev, next }
    }
}

impl<Id: Copy + Eq> Default for FocusManager<Id> {
    fn default() -> Self {
        Self::new()
    }
}
