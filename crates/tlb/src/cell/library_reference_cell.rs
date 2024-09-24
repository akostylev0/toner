use crate::cell::CellBehavior;
use crate::cell::HigherHash;
use crate::cell_type::CellType;
use crate::level_mask::LevelMask;
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use std::sync::Arc;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub data: BitVec<u8, Msb0>,
}

impl CellBehavior for LibraryReferenceCell {
    #[inline]
    fn as_type(&self) -> CellType {
        CellType::LibraryReference
    }

    #[inline]
    fn data(&self) -> &BitVec<u8, Msb0> {
        self.data.as_ref()
    }

    #[inline]
    fn references(&self) -> &[Arc<Cell>] {
        &[]
    }

    #[inline]
    fn level(&self) -> u8 {
        0
    }

    #[inline]
    fn max_depth(&self) -> u16 {
        0
    }
}

impl HigherHash for LibraryReferenceCell {
    #[inline]
    fn level_mask(&self) -> LevelMask {
        LevelMask::default()
    }

    #[inline]
    fn higher_hash(&self, _: u8) -> [u8; 32] {
        self.data
            .as_raw_slice()
            .try_into()
            .expect("invalid hash length")
    }

    #[inline]
    fn depth(&self, _: u8) -> u16 {
        0
    }
}
