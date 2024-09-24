use crate::cell::higher_hash::HigherHash;
use crate::cell::CellBehavior;
use crate::de::{CellDeserialize, CellParser, CellParserError};
use crate::level_mask::LevelMask;
use crate::Cell;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use std::mem;
use std::sync::Arc;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct LibraryReferenceCell {
    pub data: BitVec<u8, Msb0>,
}

impl CellBehavior for LibraryReferenceCell {
    fn new(data: BitVec<u8, Msb0>, _: Vec<Arc<Cell>>) -> Self {
        Self { data }
    }

    #[inline]
    #[must_use]
    fn parser(&self) -> CellParser<'_, Self> {
        CellParser::new(self.level(), self.data.as_bitslice(), &[])
    }
}

impl<'de> CellDeserialize<'de, Self> for LibraryReferenceCell {
    fn parse(
        parser: &mut CellParser<'de, LibraryReferenceCell>,
    ) -> Result<Self, CellParserError<'de, LibraryReferenceCell>> {
        let cell = LibraryReferenceCell {
            data: mem::take(&mut parser.data).to_bitvec(),
        };
        parser.ensure_empty()?;

        Ok(cell)
    }
}

impl From<LibraryReferenceCell> for Cell {
    fn from(value: LibraryReferenceCell) -> Self {
        Self::LibraryReference(value)
    }
}

impl LibraryReferenceCell {
    #[inline]
    pub fn max_depth(&self) -> u16 {
        0
    }

    #[inline]
    pub fn level(&self) -> u8 {
        0
    }
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        self.data
            .as_raw_slice()
            .try_into()
            .expect("invalid hash length")
    }
}

impl HigherHash for LibraryReferenceCell {
    fn level_mask(&self) -> LevelMask {
        LevelMask::default()
    }

    fn higher_hash(&self, _: u8) -> [u8; 32] {
        self.hash()
    }

    fn depth(&self, _: u8) -> u16 {
        0
    }
}
