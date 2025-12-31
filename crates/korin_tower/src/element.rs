use capsule_corp::ElementState;
use ginyu_force::Pose;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub tag: Pose,
    pub id: Option<Pose>,
    pub classes: SmallVec<[Pose; 4]>,
    pub attributes: FxHashMap<Pose, String>,
    pub state: ElementState,
}

impl Element {
    #[must_use]
    pub fn new(tag: Pose) -> Self {
        Self {
            tag,
            id: None,
            classes: SmallVec::new(),
            attributes: FxHashMap::default(),
            state: ElementState::empty(),
        }
    }

    #[must_use]
    pub const fn with_id(mut self, id: Pose) -> Self {
        self.id = Some(id);

        self
    }

    #[must_use]
    pub fn with_class(mut self, class: Pose) -> Self {
        self.classes.push(class);
        self
    }

    #[must_use]
    pub fn with_attribute(mut self, name: Pose, value: impl Into<String>) -> Self {
        self.attributes.insert(name, value.into());
        self
    }

    pub const fn set_id(&mut self, id: Option<Pose>) {
        self.id = id;
    }

    pub fn set_classes(&mut self, classes: SmallVec<[Pose; 4]>) {
        self.classes = classes;
    }

    pub fn set_attributes(&mut self, attributes: FxHashMap<Pose, String>) {
        self.attributes = attributes;
    }

    pub fn add_class(&mut self, class: Pose) {
        if !self.classes.contains(&class) {
            self.classes.push(class);
        }
    }

    #[must_use]
    pub fn has_class(&self, name: &str) -> bool {
        self.classes.contains(&Pose::from(name))
    }

    pub fn remove_class(&mut self, class: Pose) {
        self.classes.retain(|c| *c != class);
    }

    pub fn set_attribute(&mut self, name: Pose, value: impl Into<String>) {
        self.attributes.insert(name, value.into());
    }

    pub fn remove_attribute(&mut self, name: Pose) {
        self.attributes.remove(&name);
    }

    pub fn get_attribute(&self, name: Pose) -> Option<&str> {
        self.attributes.get(&name).map(String::as_str)
    }

    pub const fn set_state(&mut self, state: ElementState) {
        self.state = state;
    }

    pub fn add_state(&mut self, state: ElementState) {
        self.state.insert(state);
    }

    pub fn remove_state(&mut self, state: ElementState) {
        self.state.remove(state);
    }
}
