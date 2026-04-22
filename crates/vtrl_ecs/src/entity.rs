use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    #[inline]
    pub fn new(id: u32, generation: u32) -> Self {
        Entity { id, generation }
    }
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Both fields must contribute to the hash for HashMap usage
        state.write_u32(self.id);
        state.write_u32(self.generation);
    }
}
