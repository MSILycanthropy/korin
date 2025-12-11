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
            let prev = self.index;
            self.index = position;

            tracing::debug!(from = prev, to = position, "focus");
            return true;
        }

        tracing::debug!("focus failed: id not in order");
        false
    }

    pub fn focus_next(&mut self) -> FocusChange<Id> {
        if self.order.is_empty() {
            tracing::debug!("focus_next: empty order");
            return FocusChange::EMPTY;
        }

        let to = (self.index + 1) % self.len();

        self.change_focus(to)
    }

    pub fn focus_prev(&mut self) -> FocusChange<Id> {
        if self.order.is_empty() {
            tracing::debug!("focus_prev: empty order");
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
        let from = self.index;

        self.index = to;

        let next = self.focused();

        tracing::debug!(from, to, "change_focus");

        FocusChange { prev, next }
    }
}

impl<Id: Copy + Eq> Default for FocusManager<Id> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_is_empty() {
        let focus_manager: FocusManager<u32> = FocusManager::new();
        assert!(focus_manager.is_empty());
        assert!(focus_manager.focused().is_none());
    }

    #[test]
    fn set_order_updates_focusable_items() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);

        assert_eq!(focus_manager.len(), 3);
        assert!(!focus_manager.is_empty());
    }

    #[test]
    fn focused_returns_first_after_set_order() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![10, 20, 30]);

        assert_eq!(focus_manager.focused(), Some(10));
    }

    #[test]
    fn focus_next_cycles_forward() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);

        assert_eq!(focus_manager.focused(), Some(1));

        focus_manager.focus_next();
        assert_eq!(focus_manager.focused(), Some(2));

        focus_manager.focus_next();
        assert_eq!(focus_manager.focused(), Some(3));

        focus_manager.focus_next();
        assert_eq!(focus_manager.focused(), Some(1));
    }

    #[test]
    fn focus_prev_cycles_backward() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);

        assert_eq!(focus_manager.focused(), Some(1));

        focus_manager.focus_prev();
        assert_eq!(focus_manager.focused(), Some(3));

        focus_manager.focus_prev();
        assert_eq!(focus_manager.focused(), Some(2));

        focus_manager.focus_prev();
        assert_eq!(focus_manager.focused(), Some(1));
    }

    #[test]
    fn focus_next_on_empty_returns_empty_change() {
        let mut focus_manager: FocusManager<u32> = FocusManager::new();
        let change = focus_manager.focus_next();

        assert!(!change.relevant());
        assert!(change.prev().is_none());
        assert!(change.next().is_none());
    }

    #[test]
    fn focus_prev_on_empty_returns_empty_change() {
        let mut focus_manager: FocusManager<u32> = FocusManager::new();
        let change = focus_manager.focus_prev();

        assert!(!change.relevant());
    }

    #[test]
    fn focus_change_relevant_when_different() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2]);

        let change = focus_manager.focus_next();
        assert!(change.relevant());
        assert_eq!(change.prev(), Some(1));
        assert_eq!(change.next(), Some(2));
    }

    #[test]
    fn focus_change_not_relevant_for_single_item() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1]);

        let change = focus_manager.focus_next();
        assert!(!change.relevant());
        assert_eq!(change.prev(), Some(1));
        assert_eq!(change.next(), Some(1));
    }

    #[test]
    fn focus_by_id_works() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![10, 20, 30]);

        assert!(focus_manager.focus(30));
        assert_eq!(focus_manager.focused(), Some(30));
    }

    #[test]
    fn focus_by_id_fails_for_unknown() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);

        assert!(!focus_manager.focus(99));
        assert_eq!(focus_manager.focused(), Some(1));
    }

    #[test]
    fn is_focused_works() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);

        assert!(focus_manager.is_focused(1));
        assert!(!focus_manager.is_focused(2));

        focus_manager.focus_next();
        assert!(!focus_manager.is_focused(1));
        assert!(focus_manager.is_focused(2));
    }

    #[test]
    fn set_order_preserves_current_focus_if_present() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);
        focus_manager.focus(2);

        focus_manager.set_order(vec![3, 2, 1]);
        assert_eq!(focus_manager.focused(), Some(2));
    }

    #[test]
    fn set_order_resets_remains_at_index_if_current_is_not_present() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);
        focus_manager.focus(2);

        focus_manager.set_order(vec![10, 20, 30]);
        assert_eq!(focus_manager.focused(), Some(20));
    }

    #[test]
    fn get_pos_returns_correct_index() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![10, 20, 30]);

        assert_eq!(focus_manager.get_pos(10), Some(0));
        assert_eq!(focus_manager.get_pos(20), Some(1));
        assert_eq!(focus_manager.get_pos(30), Some(2));
        assert_eq!(focus_manager.get_pos(99), None);
    }

    #[test]
    fn clear_empties_manager() {
        let mut focus_manager = FocusManager::new();
        focus_manager.set_order(vec![1, 2, 3]);

        focus_manager.clear();
        assert!(focus_manager.is_empty());
        assert!(focus_manager.focused().is_none());
    }
}
