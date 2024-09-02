pub mod ordinary;
pub mod library_reference;

pub use self::ordinary::*;
pub use self::library_reference::*;

use crate::de::r#as::CellDeserializeAs;
use crate::de::{CellDeserialize, CellParser, CellParserError};
use crate::ser::{CellBuilderError, CellSerialize, OrdinaryCellBuilder};
use std::mem;
use std::sync::Arc;
use tlbits::{Error, StringError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cell {
    Ordinary(OrdinaryCell),
    LibraryReference(LibraryReference),
}

impl Cell {
    pub fn references(&self) -> &[Arc<Cell>] {
        match self {
            Cell::Ordinary(ordinary) => ordinary.references.as_slice(),
            Cell::LibraryReference(_) => &[],
        }
    }
}

impl Cell {
    pub fn parse_fully<'de, T>(&'de self) -> Result<T, CellParserError<'de>>
    where
        T: CellDeserialize<'de>,
    {
        match self {
            Cell::Ordinary(cell) => cell.parse_fully(),
            Cell::LibraryReference(_) => todo!()
        }
    }

    #[inline]
    pub fn parse_fully_as<'de, T, As>(&'de self) -> Result<T, CellParserError<'de>>
    where
        As: CellDeserializeAs<'de, T> + ?Sized,
    {
        match self {
            Cell::Ordinary(cell) => cell.parse_fully_as::<T, As>(),
            Cell::LibraryReference(_) => todo!()
        }
    }
}

impl Cell {
    pub fn as_ordinary(&self) -> Result<&OrdinaryCell, StringError> {
        match self {
            Cell::Ordinary(ordinary) => Ok(ordinary),
            _ => Err(Error::custom("expected ordinary cell")),
        }
    }
}

impl Cell {
    pub fn level(&self) -> u8 {
        match self {
            Cell::Ordinary(ordinary) => ordinary.level(),
            Cell::LibraryReference(_) => 0
        }
    }

    pub fn max_depth(&self) -> u16 {
        match self {
            Cell::Ordinary(ordinary) => ordinary.max_depth(),
            Cell::LibraryReference(_) => 0
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        match self {
            Cell::Ordinary(ordinary) => ordinary.hash(),
            Cell::LibraryReference(library_reference) => library_reference.repr_hash,
        }
    }
}

impl CellSerialize for Cell {
    #[inline]
    fn store(&self, builder: &mut OrdinaryCellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Cell::Ordinary(cell) => cell.store(builder),
            Cell::LibraryReference(_) => todo!(),
        }
    }
}

impl<'de> CellDeserialize<'de> for Cell {
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Cell::Ordinary(OrdinaryCell {
            data: mem::take(&mut parser.data).to_bitvec(),
            references: mem::take(&mut parser.references).to_vec(),
        }))
    }
}
