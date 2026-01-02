//! A small library for interned strings. Think a shittier `string-cache`.
//!
//! Static poses are defined in `poses.txt` and are free to create.
//! Dynamic poses (like custom property names) are interned at runtime.
//!
//! ```
//! use ginyu_force::{Pose, pose};
//!
//! // Static pose - no allocation
//! let color = pose!("color");
//!
//! // Dynamic pose - interned at runtime
//! let custom = Pose::from("--my-custom-prop");
//!
//! // Poses are Copy and cheap to compare
//! let color2 = color;
//! assert_eq!(color, color2);
//! ```

mod interner;

use std::{cmp::Ordering, fmt, hash::Hash};

include!(concat!(env!("OUT_DIR"), "/static_poses.rs"));

/// An interned string.
///
/// This is `Copy` and cheap to compare (O(1) equality).
/// Static poses (from `pose!()` macro) are zero-cost.
/// Dynamic poses are interned in a global table.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pose(u32);

impl Pose {
    const DYNAMIC_BIT: u32 = 1 << 31;

    /// Create a pose from a static index. Used by the `pose!` macro.
    #[inline]
    #[must_use]
    pub const fn from_static(index: u32) -> Self {
        debug_assert!(index < Self::DYNAMIC_BIT);
        Self(index)
    }

    #[inline]
    #[must_use]
    pub fn from_dynamic(index: u32) -> Self {
        debug_assert!(index < Self::DYNAMIC_BIT);
        Self(index | Self::DYNAMIC_BIT)
    }

    /// Get the string value of this pose.
    #[inline]
    #[must_use]
    pub fn as_str(self) -> &'static str {
        if self.is_static() {
            STATIC_STRINGS[self.index() as usize]
        } else {
            interner::get(self.index())
        }
    }

    /// Check if this is a static (compile-time known) pose.
    #[inline]
    #[must_use]
    pub const fn is_static(self) -> bool {
        self.0 & Self::DYNAMIC_BIT == 0
    }

    #[inline]
    const fn index(self) -> u32 {
        self.0 & !Self::DYNAMIC_BIT
    }
}

impl From<&str> for Pose {
    fn from(str: &str) -> Self {
        if let Some(index) = static_pose_index(str) {
            return Self::from_static(index);
        }

        let index = interner::intern(str);
        Self::from_dynamic(index)
    }
}

impl From<String> for Pose {
    fn from(string: String) -> Self {
        Self::from(string.as_str())
    }
}

impl fmt::Debug for Pose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pose({:?})", self.as_str())
    }
}

impl fmt::Display for Pose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<str> for Pose {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Pose {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialOrd for Pose {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pose {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            return Ordering::Equal;
        }

        self.as_str().cmp(other.as_str())
    }
}

impl Default for Pose {
    fn default() -> Self {
        Self::from("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_pose() {
        let p = pose!("color");
        assert!(p.is_static());
        assert_eq!(p.as_str(), "color");
    }

    #[test]
    fn dynamic_pose() {
        let p = Pose::from("some-random-string-not-in-static");
        assert!(!p.is_static());
        assert_eq!(p.as_str(), "some-random-string-not-in-static");
    }

    #[test]
    fn static_via_from() {
        let p = Pose::from("color");
        assert!(p.is_static());
    }

    #[test]
    fn equality() {
        let a = pose!("color");
        let b = pose!("color");
        let c = Pose::from("color");

        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn dynamic_equality() {
        let a = Pose::from("xyz-not-static");
        let b = Pose::from("xyz-not-static");

        assert_eq!(a, b);
    }

    #[test]
    fn copy() {
        let a = pose!("color");
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn compare_to_str() {
        let p = pose!("color");
        assert!(p == "color");
    }

    #[test]
    fn ordering() {
        let a = Pose::from("apple");
        let b = Pose::from("banana");
        let c = Pose::from("apple");

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a.cmp(&c), Ordering::Equal);
    }

    #[test]
    fn ordering_same_pose_fast_path() {
        let a = pose!("color");
        let b = pose!("color");
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn size() {
        assert_eq!(std::mem::size_of::<Pose>(), 4);
    }
}
