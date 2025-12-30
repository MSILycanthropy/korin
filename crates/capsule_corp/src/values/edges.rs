#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Clone> Edges<T> {
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn all(value: T) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    pub fn symmetric(vertical: T, horizontal: T) -> Self {
        Self {
            top: vertical.clone(),
            bottom: vertical,
            left: horizontal.clone(),
            right: horizontal,
        }
    }

    pub fn three(top: T, horizontal: T, bottom: T) -> Self {
        Self {
            top,
            left: horizontal.clone(),
            right: horizontal,
            bottom,
        }
    }

    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> Edges<U> {
        Edges {
            top: f(&self.top),
            right: f(&self.right),
            bottom: f(&self.bottom),
            left: f(&self.left),
        }
    }
}

impl<T: Default> Default for Edges<T> {
    fn default() -> Self {
        Self {
            top: T::default(),
            right: T::default(),
            bottom: T::default(),
            left: T::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_all() {
        let e = Edges::all(5);
        assert_eq!(e.top, 5);
        assert_eq!(e.right, 5);
        assert_eq!(e.bottom, 5);
        assert_eq!(e.left, 5);
    }

    #[test]
    fn edges_symmetric() {
        let e = Edges::symmetric(1, 2);
        assert_eq!(e.top, 1);
        assert_eq!(e.bottom, 1);
        assert_eq!(e.left, 2);
        assert_eq!(e.right, 2);
    }

    #[test]
    fn edges_three() {
        let e = Edges::three(1, 2, 3);
        assert_eq!(e.top, 1);
        assert_eq!(e.left, 2);
        assert_eq!(e.right, 2);
        assert_eq!(e.bottom, 3);
    }

    #[test]
    fn edges_new() {
        let e = Edges::new(1, 2, 3, 4);
        assert_eq!(e.top, 1);
        assert_eq!(e.right, 2);
        assert_eq!(e.bottom, 3);
        assert_eq!(e.left, 4);
    }

    #[test]
    fn edges_map() {
        let e = Edges::all(5);
        let doubled = e.map(|x| x * 2);
        assert_eq!(doubled.top, 10);
        assert_eq!(doubled.right, 10);
    }

    #[test]
    fn edges_default() {
        let e: Edges<i32> = Edges::default();
        assert_eq!(e.top, 0);
    }
}
