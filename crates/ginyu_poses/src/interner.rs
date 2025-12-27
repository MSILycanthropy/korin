use std::sync::OnceLock;

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

static INTERNER: OnceLock<RwLock<Interner>> = OnceLock::new();

fn global() -> &'static RwLock<Interner> {
    INTERNER.get_or_init(|| RwLock::new(Interner::new()))
}

pub fn intern(str: &str) -> u32 {
    {
        let interner = global().read();
        if let Some(&index) = interner.lookup.get(str) {
            return index;
        }
    }

    let mut interner = global().write();

    if let Some(&index) = interner.lookup.get(str) {
        return index;
    }

    interner.insert(str)
}

/// Get a string by its dynamic index.
///
/// # Panics
/// Panics if the index is invali
pub fn get(index: u32) -> &'static str {
    let interner = global().read();
    interner
        .strings
        .get(index as usize)
        .expect("invalid pose index")
}

pub struct Interner {
    strings: Vec<&'static str>,

    lookup: FxHashMap<&'static str, u32>,
}

impl Interner {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            lookup: FxHashMap::default(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]  
    fn insert(&mut self, str: &str) -> u32 {
        let leaked: &'static str = Box::leak(str.into());
        let index = self.strings.len() as u32;
        self.strings.push(leaked);
        self.lookup.insert(leaked, index);
        index
    }
}
