use crate::Cell;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use sha2::{Digest, Sha256};
use std::mem;
use std::sync::Arc;
use crate::cell::higher_hash::HigherHash;
use crate::cell::CellBehavior;
use crate::cell_type::CellType;
use crate::de::{CellDeserialize, CellParser, CellParserError};
use crate::level_mask::LevelMask;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrunedBranchCell {
    pub data: BitVec<u8, Msb0>,
}

impl CellBehavior for PrunedBranchCell {
    fn new(data: BitVec<u8, Msb0>, _: Vec<Arc<Cell>>) -> Self {
        Self { data }
    }

    #[inline]
    #[must_use]
    fn parser(&self) -> CellParser<'_, Self> {
        CellParser::new(self.level(), self.data.as_bitslice(), &[])
    }
}

impl<'de> CellDeserialize<'de, Self> for PrunedBranchCell {
    fn parse(
        parser: &mut CellParser<'de, PrunedBranchCell>,
    ) -> Result<Self, CellParserError<'de, PrunedBranchCell>> {
        let cell = PrunedBranchCell {
            data: mem::take(&mut parser.data).to_bitvec(),
        };

        parser.ensure_empty()?;
        Ok(cell)
    }
}

impl From<PrunedBranchCell> for Cell {
    fn from(value: PrunedBranchCell) -> Self {
        Self::PrunedBranch(value)
    }
}

impl HigherHash for PrunedBranchCell {
    fn level_mask(&self) -> LevelMask {
        LevelMask::new(
            self.data
                .as_raw_slice()
                .first()
                .cloned()
                .expect("invalid data length"),
        )
    }

    fn higher_hash(&self, level: u8) -> [u8; 32] {
        if self.level_mask().contains(level) {
            self.data.as_raw_slice()[1 + (32 * level) as usize..1 + (32 * (level + 1)) as usize]
                .try_into()
                .expect("invalid data length")
        } else {
            let mut hasher = Sha256::new();
            hasher.update([
                self.refs_descriptor(),
                self.bits_descriptor(),
                CellType::PrunedBranch as u8,
            ]);
            hasher.update(self.data.as_raw_slice());

            hasher.finalize().into()
        }
    }

    fn depth(&self, level: u8) -> u16 {
        if self.level_mask().contains(level) {
            let view = self.data.as_raw_slice();
            u16::from_be_bytes([
                view[(1 + 32 * self.level_mask().as_level() + 2 * level) as usize],
                view[(1 + 32 * self.level_mask().as_level() + 2 * level + 1) as usize],
            ])
        } else {
            0
        }
    }
}

impl PrunedBranchCell {
    pub fn max_depth(&self) -> u16 {
        self.depths().into_iter().max().unwrap_or(0)
    }

    pub fn level(&self) -> u8 {
        self.data.as_raw_slice().first().cloned().unwrap()
    }

    fn depths(&self) -> Vec<u16> {
        let level = self.level();
        let depths = &self.data.as_raw_slice()
            [(1 + 32 * level) as usize..(1 + 32 * level + 2 * level) as usize];

        depths
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes(c.try_into().unwrap()))
            .collect()
    }

    #[inline]
    fn refs_descriptor(&self) -> u8 {
        8 + 32 * self.level()
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.data.len() + 8;

        (b / 8) as u8 + ((b + 7) / 8) as u8
    }
}
