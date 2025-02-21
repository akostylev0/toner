use crate::cell::CellBehavior;
use crate::cell::HigherHash;
use crate::cell_type::CellType;
use crate::level_mask::LevelMask;
use crate::Cell;
use bitvec::order::Msb0;
use std::sync::Arc;
use bitvec::array::BitArray;
use bitvec::slice::BitSlice;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub data: BitArray<[u8; 32], Msb0>,
}

impl LibraryReferenceCell {
    pub fn from_bitslice(bits: &BitSlice<u8, Msb0>) -> Self {
        let mut data = BitArray::default();
        data.copy_from_bitslice(bits);

        Self { data }
    }
}

impl CellBehavior for LibraryReferenceCell {
    #[inline]
    fn as_type(&self) -> CellType {
        CellType::LibraryReference
    }

    #[inline]
    fn data(&self) -> &BitSlice<u8, Msb0> {
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
        self.data.into_inner()
    }

    #[inline]
    fn depth(&self, _: u8) -> u16 {
        0
    }
}
