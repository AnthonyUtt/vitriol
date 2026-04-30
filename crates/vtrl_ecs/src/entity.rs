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

    pub fn as_u64(self) -> u64 {
        ((self.id as u64) << 32) + self.generation as u64
    }

    pub fn from_u64(value: u64) -> Self {
        let generation = (value & (u32::MAX as u64)) as u32;
        let id = (value >> 32) as u32;
        Self { id, generation }
    }
}

impl Hash for Entity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Both fields must contribute to the hash for HashMap usage
        state.write_u32(self.id);
        state.write_u32(self.generation);
    }
}
