mod library_reference_cell;
mod merkle_proof_cell;
mod merkle_update_cell;
mod ordinary_cell;
mod pruned_branch_cell;

use ambassador::{delegatable_trait, Delegate};
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use core::{
    fmt::{self, Debug},
    hash::Hash,
};
use std::sync::Arc;

pub use crate::cell::library_reference_cell::LibraryReferenceCell;
pub use crate::cell::merkle_proof_cell::MerkleProofCell;
pub use crate::cell::merkle_update_cell::MerkleUpdateCell;
pub use crate::cell::ordinary_cell::OrdinaryCell;
pub use crate::cell::pruned_branch_cell::*;
use crate::cell_type::CellType;
use crate::level_mask::LevelMask;
use crate::{
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellDeserialize, CellParser, CellParserError,
    },
    ser::CellBuilder,
};

#[delegatable_trait]
pub trait CellBehavior {
    fn as_type(&self) -> CellType;

    fn data(&self) -> &BitVec<u8, Msb0>;

    fn references(&self) -> &[Arc<Cell>];

    /// See [Cell level](https://docs.ton.org/develop/data-formats/cell-boc#cell-level)
    fn level(&self) -> u8;

    fn max_depth(&self) -> u16;

    #[inline]
    fn len(&self) -> usize {
        self.data().len()
    }

    #[inline]
    fn as_raw_slice(&self) -> &[u8] {
        self.data().as_raw_slice()
    }

    #[inline]
    fn as_bitslice(&self) -> &BitSlice<u8, Msb0> {
        self.data().as_bitslice()
    }

    /// Returns whether this cell has no data and zero references.
    #[inline]
    fn is_empty(&self) -> bool {
        self.data().is_empty() && self.references().is_empty()
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn refs_descriptor(&self, level_mask: LevelMask) -> u8 {
        self.references().len() as u8
            | if self.as_type().is_exotic() {
                1 << 3
            } else {
                0
            }
            | (level_mask.as_u8() << 5)
    }

    /// See [Cell serialization](https://docs.ton.org/develop/data-formats/cell-boc#cell-serialization)
    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.len() + if self.as_type().is_exotic() { 8 } else { 0 };

        (b / 8) as u8 + ((b + 7) / 8) as u8
    }
}

#[delegatable_trait]
pub trait HigherHash {
    fn level_mask(&self) -> LevelMask;

    fn higher_hash(&self, level: u8) -> [u8; 32];

    fn depth(&self, level: u8) -> u16;

    /// Calculates [standard Cell representation hash](https://docs.ton.org/develop/data-formats/cell-boc#cell-hash)
    #[inline]
    fn representation_hash(&self) -> [u8; 32] {
        self.higher_hash(0)
    }
}

/// A [Cell](https://docs.ton.org/develop/data-formats/cell-boc#cell).
#[derive(Clone, PartialEq, Eq, Hash, Delegate)]
#[delegate(HigherHash)]
#[delegate(CellBehavior)]
pub enum Cell {
    Ordinary(OrdinaryCell),
    LibraryReference(LibraryReferenceCell),
    PrunedBranch(PrunedBranchCell),
    MerkleProof(MerkleProofCell),
    MerkleUpdate(MerkleUpdateCell),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Ordinary(OrdinaryCell::default())
    }
}

impl Cell {
    pub fn as_library_reference(&self) -> Option<&LibraryReferenceCell> {
        match self {
            Cell::LibraryReference(reference) => Some(reference),
            _ => None,
        }
    }

    pub fn as_merkle_proof(&self) -> Option<&MerkleProofCell> {
        match self {
            Cell::MerkleProof(proof) => Some(proof),
            _ => None,
        }
    }

    pub fn as_merkle_update(&self) -> Option<&MerkleUpdateCell> {
        match self {
            Cell::MerkleUpdate(update) => Some(update),
            _ => None,
        }
    }

    pub fn as_pruned_branch(&self) -> Option<&PrunedBranchCell> {
        match self {
            Cell::PrunedBranch(branch) => Some(branch),
            _ => None,
        }
    }

    pub fn as_ordinary(&self) -> Option<&OrdinaryCell> {
        match self {
            Cell::Ordinary(cell) => Some(cell),
            _ => None,
        }
    }
}

impl Cell {
    /// Create new [`CellBuilder`]
    #[inline]
    #[must_use]
    pub const fn builder() -> CellBuilder {
        CellBuilder::new()
    }

    /// Return [`CellParser`] for this cell
    #[inline]
    #[must_use]
    pub fn parser(&self) -> CellParser<'_> {
        CellParser::new(
            self.as_type(),
            self.level(),
            self.as_bitslice(),
            self.references(),
        )
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse()`](CellParser::parse)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully<'de, T>(&'de self) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        let mut parser = self.parser();
        let v = parser.parse()?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_with()`](CellParser::parse_with)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_with<'de, T>(&'de self, args: T::Args) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserializeWithArgs<'de>,
    {
        let mut parser = self.parser();
        let v = parser.parse_with(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as()`](CellParser::parse_as)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as<'de, T, As>(&'de self) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as::<T, As>()?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as_with()`](CellParser::parse_as_with)[`.ensure_empty()`](CellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as_with<'de, T, As>(
        &'de self,
        args: As::Args,
    ) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as_with::<T, As>(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}:L{}:R{}:D{}:",
            self.as_type(),
            self.level(),
            self.references().len(),
            self.max_depth()
        )?;

        if f.alternate() {
            write!(f, "{}[0b", self.len())?;
            for bit in self.as_bitslice() {
                write!(f, "{}", if *bit { '1' } else { '0' })?;
            }
            write!(f, "]")?;
        } else {
            let bits_len = self.len();
            let data = self.as_raw_slice();
            write!(f, "{}[0x{}]", bits_len, hex::encode_upper(data))?;
        }
        if self.references().is_empty() {
            return Ok(());
        }
        write!(f, " -> ")?;
        f.debug_set().entries(self.references()).finish()
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{
        bits::{r#as::NBits, ser::BitWriterExt},
        r#as::{Data, Ref},
        ser::{r#as::CellSerializeWrapAsExt, CellSerializeExt},
        tests::assert_store_parse_as_eq,
        HigherHash,
    };

    use super::*;

    #[test]
    fn zero_depth() {
        assert_eq!(().to_cell().unwrap().max_depth(), 0)
    }

    #[test]
    fn max_depth() {
        let cell = (
            ().wrap_as::<Ref>(),
            (().wrap_as::<Ref>(), ().wrap_as::<Ref<Ref>>())
                .wrap_as::<Ref>()
                .wrap_as::<Ref>(),
            ((), ()),
        )
            .to_cell()
            .unwrap();
        assert_eq!(cell.max_depth(), 4)
    }

    #[test]
    fn cell_serde() {
        assert_store_parse_as_eq::<
            _,
            (
                Data<NBits<1>>,
                Ref<Data<NBits<24>>>,
                Ref<(Data<NBits<7>>, Ref<Data<NBits<24>>>)>,
            ),
        >((0b1, 0x0AAAAA, (0x7F, 0x0AAAAA)));
    }

    #[test]
    fn hash_no_refs() {
        let mut builder = Cell::builder();
        builder.pack_as::<_, NBits<32>>(0x0000000F).unwrap();
        let cell = builder.into_cell();

        assert_eq!(
            cell.representation_hash(),
            hex!("57b520dbcb9d135863fc33963cde9f6db2ded1430d88056810a2c9434a3860f9")
        );
    }

    #[test]
    fn hash_with_refs() {
        let mut builder = Cell::builder();
        builder
            .store_as::<_, Data<NBits<24>>>(0x00000B)
            .unwrap()
            .store_reference_as::<_, Data>(0x0000000F_u32)
            .unwrap()
            .store_reference_as::<_, Data>(0x0000000F_u32)
            .unwrap();
        let cell = builder.into_cell();

        assert_eq!(
            cell.representation_hash(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}
