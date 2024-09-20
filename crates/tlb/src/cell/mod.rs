use core::{
    fmt::{self, Debug},
    hash::Hash,
};
use std::sync::Arc;

use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;

use tlbits::Error;

use crate::cell::higher_hash::HigherHash;
pub use crate::cell::library_reference_cell::LibraryReferenceCell;
pub use crate::cell::merkle_proof_cell::MerkleProofCell;
pub use crate::cell::merkle_update_cell::MerkleUpdateCell;
pub use crate::cell::ordinary_cell::OrdinaryCell;
pub use crate::cell::pruned_branch_cell::*;
use crate::cell_type::CellType;
use crate::de::{CellParser, CellParserError};
use crate::level_mask::LevelMask;
use crate::{
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellDeserialize,
    },
    ser::CellBuilder,
};

pub mod higher_hash;
mod library_reference_cell;
mod merkle_proof_cell;
mod merkle_update_cell;
mod ordinary_cell;
mod pruned_branch_cell;

/// A [Cell](https://docs.ton.org/develop/data-formats/cell-boc#cell).
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Cell {
    Ordinary(OrdinaryCell),
    LibraryReference(LibraryReferenceCell),
    PrunedBranch(PrunedBranchCell),
    MerkleProof(MerkleProofCell),
    MerkleUpdate(MerkleUpdateCell),
}

pub trait CellBehavior
where
    Self: Sized,
{
    fn parser(&self) -> CellParser<'_, Self>;

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse()`](OrdinaryCellParser::parse)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    fn parse_fully<'de, T>(&'de self) -> Result<T, CellParserError<'de, Self>>
    where
        T: CellDeserialize<'de, Self>,
    {
        let mut parser = self.parser();
        let v = parser.parse()?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_with()`](OrdinaryCellParser::parse_with)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    fn parse_fully_with<'de, T>(&'de self, args: T::Args) -> Result<T, CellParserError<'de, Self>>
    where
        T: CellDeserializeWithArgs<'de, Self>,
    {
        let mut parser = self.parser();
        let v = parser.parse_with(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as()`](OrdinaryCellParser::parse_as)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    fn parse_fully_as<'de, T, As>(&'de self) -> Result<T, CellParserError<'de, Self>>
    where
        As: CellDeserializeAs<'de, T, Self> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as::<T, As>()?;
        parser.ensure_empty()?;
        Ok(v)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as_with()`](OrdinaryCellParser::parse_as_with)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    fn parse_fully_as_with<'de, T, As>(
        &'de self,
        args: As::Args,
    ) -> Result<T, CellParserError<'de, Self>>
    where
        As: CellDeserializeAsWithArgs<'de, T, Self> + ?Sized,
    {
        let mut parser = self.parser();
        let v = parser.parse_as_with::<T, As>(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Ordinary(OrdinaryCell::default())
    }
}

impl HigherHash for Cell {
    fn level_mask(&self) -> LevelMask {
        match self {
            Cell::Ordinary(inner) => inner.level_mask(),
            Cell::LibraryReference(inner) => inner.level_mask(),
            Cell::PrunedBranch(inner) => inner.level_mask(),
            Cell::MerkleProof(inner) => inner.level_mask(),
            Cell::MerkleUpdate(inner) => inner.level_mask(),
        }
    }
    fn higher_hash(&self, level: u8) -> [u8; 32] {
        match self {
            Cell::Ordinary(inner) => inner.higher_hash(level),
            Cell::LibraryReference(inner) => inner.higher_hash(level),
            Cell::PrunedBranch(inner) => inner.higher_hash(level),
            Cell::MerkleProof(inner) => inner.higher_hash(level),
            Cell::MerkleUpdate(inner) => inner.higher_hash(level),
        }
    }

    fn depth(&self, level: u8) -> u16 {
        match self {
            Cell::Ordinary(inner) => inner.depth(level),
            Cell::LibraryReference(inner) => inner.depth(level),
            Cell::PrunedBranch(inner) => inner.depth(level),
            Cell::MerkleProof(inner) => inner.depth(level),
            Cell::MerkleUpdate(inner) => inner.depth(level),
        }
    }
}

impl Cell {
    pub fn as_type(&self) -> CellType {
        match self {
            Cell::Ordinary(_) => CellType::Ordinary,
            Cell::LibraryReference(_) => CellType::LibraryReference,
            Cell::PrunedBranch(_) => CellType::PrunedBranch,
            Cell::MerkleProof(_) => CellType::MerkleProof,
            Cell::MerkleUpdate(_) => CellType::MerkleUpdate,
        }
    }

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

    #[inline]
    pub fn data(&self) -> &BitVec<u8, Msb0> {
        match self {
            Cell::Ordinary(OrdinaryCell { data, .. }) => data,
            Cell::LibraryReference(LibraryReferenceCell { data }) => data,
            Cell::PrunedBranch(PrunedBranchCell { data, .. }) => data,
            Cell::MerkleProof(MerkleProofCell { data, .. }) => data,
            Cell::MerkleUpdate(MerkleUpdateCell { data, .. }) => data,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data().len()
    }

    #[inline]
    pub fn as_raw_slice(&self) -> &[u8] {
        self.data().as_raw_slice()
    }

    #[inline]
    pub fn as_bitslice(&self) -> &BitSlice<u8, Msb0> {
        self.data().as_bitslice()
    }

    #[inline]
    pub fn references(&self) -> &[Arc<Self>] {
        match self {
            Cell::Ordinary(OrdinaryCell { references, .. }) => references,
            Cell::LibraryReference(_) => &[],
            Cell::PrunedBranch(_) => &[],
            Cell::MerkleProof(MerkleProofCell { references, .. }) => references,
            Cell::MerkleUpdate(MerkleUpdateCell { references, .. }) => references,
        }
    }

    /// Returns whether this cell has no data and zero references.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().is_empty() && self.references().is_empty()
    }

    #[inline]
    fn data_bytes(&self) -> (usize, &[u8]) {
        (self.len(), self.as_raw_slice())
    }

    /// See [Cell level](https://docs.ton.org/develop/data-formats/cell-boc#cell-level)
    #[inline]
    pub fn level(&self) -> u8 {
        match self {
            Cell::LibraryReference(inner) => inner.level(),
            Cell::Ordinary(inner) => inner.level(),
            Cell::PrunedBranch(inner) => inner.level(),
            Cell::MerkleProof(inner) => inner.level(),
            Cell::MerkleUpdate(inner) => inner.level(),
        }
    }

    #[inline]
    fn max_depth(&self) -> u16 {
        match self {
            Cell::LibraryReference(inner) => inner.max_depth(),
            Cell::Ordinary(inner) => inner.max_depth(),
            Cell::PrunedBranch(inner) => inner.max_depth(),
            Cell::MerkleProof(inner) => inner.max_depth(),
            Cell::MerkleUpdate(inner) => inner.max_depth(),
        }
    }

    /// Calculates [standard Cell representation hash](https://docs.ton.org/develop/data-formats/cell-boc#cell-hash)
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        self.higher_hash(0)
    }

    #[inline]
    pub fn parser<'a, C>(&'a self) -> Result<CellParser<'a, C>, CellParserError<'a, Self>>
    where
        &'a Cell: TryInto<&'a C>,
        C: CellBehavior + 'a,
    {
        Ok(self
            .try_into()
            .map_err(|_| CellParserError::<C>::custom("wrong cell type"))?
            .parser())
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse()`](OrdinaryCellParser::parse)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    pub fn parse_fully<'de, T, C>(&'de self) -> Result<T, CellParserError<'de, Self>>
    where
        T: CellDeserialize<'de, C>,
        &'de Cell: TryInto<&'de C>,
        C: CellBehavior + 'de,
    {
        self.try_into()
            .map_err(|_| CellParserError::<C>::custom("wrong cell type"))?
            .parse_fully::<T>()
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_with()`](OrdinaryCellParser::parse_with)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_with<'de, T, C>(
        &'de self,
        args: T::Args,
    ) -> Result<T, CellParserError<'de, Self>>
    where
        T: CellDeserializeWithArgs<'de, C>,
        &'de Cell: TryInto<&'de C>,
        C: CellBehavior + 'de,
    {
        self.try_into()
            .map_err(|_| CellParserError::<C>::custom("wrong cell type"))?
            .parse_fully_with::<T>(args)
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as()`](OrdinaryCellParser::parse_as)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as<'de, T, As, C>(&'de self) -> Result<T, CellParserError<'de, C>>
    where
        As: CellDeserializeAs<'de, T, C> + ?Sized,
        &'de Cell: TryInto<&'de C>,
        C: CellBehavior + 'de,
    {
        self.try_into()
            .map_err(|_| CellParserError::<C>::custom("wrong cell type"))?
            .parse_fully_as::<T, As>()
    }

    /// Shortcut for [`.parser()`](Cell::parser)[`.parse_as_with()`](OrdinaryCellParser::parse_as_with)[`.ensure_empty()`](OrdinaryCellParser::ensure_empty).
    #[inline]
    pub fn parse_fully_as_with<'de, T, As, C>(
        &'de self,
        args: As::Args,
    ) -> Result<T, CellParserError<'de, C>>
    where
        As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
        &'de Cell: TryInto<&'de C>,
        C: CellBehavior + 'de,
    {
        self.try_into()
            .map_err(|_| CellParserError::<C>::custom("wrong cell type"))?
            .parse_fully_as_with::<T, As>(args)
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
            let (bits_len, data) = self.data_bytes();
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
            cell.hash(),
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
            cell.hash(),
            hex!("f345277cc6cfa747f001367e1e873dcfa8a936b8492431248b7a3eeafa8030e7")
        );
    }
}

impl TryInto<LibraryReferenceCell> for Cell {
    type Error = ();

    fn try_into(self) -> Result<LibraryReferenceCell, Self::Error> {
        if let Cell::LibraryReference(inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl TryInto<MerkleProofCell> for Cell {
    type Error = ();

    fn try_into(self) -> Result<MerkleProofCell, Self::Error> {
        if let Cell::MerkleProof(inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl TryInto<MerkleUpdateCell> for Cell {
    type Error = ();

    fn try_into(self) -> Result<MerkleUpdateCell, Self::Error> {
        if let Cell::MerkleUpdate(inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl TryInto<OrdinaryCell> for Cell {
    type Error = ();

    fn try_into(self) -> Result<OrdinaryCell, Self::Error> {
        if let Cell::Ordinary(inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl TryInto<PrunedBranchCell> for Cell {
    type Error = ();

    fn try_into(self) -> Result<PrunedBranchCell, Self::Error> {
        if let Cell::PrunedBranch(inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl<'a> TryInto<&'a LibraryReferenceCell> for &'a Cell {
    type Error = ();

    fn try_into(self) -> Result<&'a LibraryReferenceCell, Self::Error> {
        if let Cell::LibraryReference(ref inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl<'a> TryInto<&'a MerkleProofCell> for &'a Cell {
    type Error = ();

    fn try_into(self) -> Result<&'a MerkleProofCell, Self::Error> {
        if let Cell::MerkleProof(ref inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl<'a> TryInto<&'a MerkleUpdateCell> for &'a Cell {
    type Error = ();

    fn try_into(self) -> Result<&'a MerkleUpdateCell, Self::Error> {
        if let Cell::MerkleUpdate(ref inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl<'a> TryInto<&'a OrdinaryCell> for &'a Cell {
    type Error = ();

    fn try_into(self) -> Result<&'a OrdinaryCell, Self::Error> {
        if let Cell::Ordinary(ref inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}

impl<'a> TryInto<&'a PrunedBranchCell> for &'a Cell {
    type Error = ();

    fn try_into(self) -> Result<&'a PrunedBranchCell, Self::Error> {
        if let Cell::PrunedBranch(ref inner) = self {
            Ok(inner)
        } else {
            Err(())
        }
    }
}
