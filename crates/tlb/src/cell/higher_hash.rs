use crate::level_mask::LevelMask;

pub trait HigherHash {
    fn level_mask(&self) -> LevelMask;
    fn higher_hash(&self, level: u8) -> [u8; 32];
    fn depth(&self, level: u8) -> u16;
}
