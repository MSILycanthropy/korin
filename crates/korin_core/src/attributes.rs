use std::ops::{Deref, DerefMut};

use markup5ever::QualName;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Attribute {
    pub name: QualName,
    pub value: String,
}

#[derive(Clone, Debug, Default)]
pub struct Attributes {
    inner: Vec<Attribute>,
}

impl Attributes {
    #[must_use]
    pub const fn new(inner: Vec<Attribute>) -> Self {
        Self { inner }
    }

    pub fn set(&mut self, name: QualName, value: String) {
        if let Some(attr) = self.inner.iter_mut().find(|a| a.name == name) {
            attr.value.clear();
            attr.value.push_str(&value);
        } else {
            self.inner.push(Attribute { name, value });
        }
    }

    pub fn remove(&mut self, name: &QualName) -> Option<String> {
        if let Some(idx) = self.inner.iter().position(|a| a.name == *name) {
            Some(self.inner.remove(idx).value)
        } else {
            None
        }
    }
}

impl Deref for Attributes {
    type Target = Vec<Attribute>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
