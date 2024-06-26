use std::sync::Arc;

use crate::{
    bits::{
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        ser::{BitWriter, LimitWriter},
    },
    Cell, Error, ResultExt,
};

use super::{
    args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
    r#as::CellSerializeAs,
    CellSerialize,
};

type CellBitWriter = LimitWriter<BitVec<u8, Msb0>>;
pub type CellBuilderError = <CellBuilder as BitWriter>::Error;

pub struct CellBuilder {
    data: CellBitWriter,
    references: Vec<Arc<Cell>>,
}

const MAX_BITS_LEN: usize = 1023;
const MAX_REFS_COUNT: usize = 4;

impl CellBuilder {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            data: LimitWriter::new(BitVec::EMPTY, MAX_BITS_LEN),
            references: Vec::new(),
        }
    }

    #[inline]
    pub fn store<T>(&mut self, value: T) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
    {
        value.store(self)?;
        Ok(self)
    }

    #[inline]
    pub fn store_with<T>(&mut self, value: T, args: T::Args) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerializeWithArgs,
    {
        value.store_with(self, args)?;
        Ok(self)
    }

    #[inline]
    pub fn store_many<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    pub fn store_many_with<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: T::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerializeWithArgs,
        T::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_with(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    pub fn store_as<T, As>(&mut self, value: T) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        As::store_as(&value, self)?;
        Ok(self)
    }

    #[inline]
    pub fn store_as_with<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAsWithArgs<T> + ?Sized,
    {
        As::store_as_with(&value, self, args)?;
        Ok(self)
    }

    #[inline]
    pub fn store_many_as<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as::<T, As>(v)
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    pub fn store_many_as_with<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAsWithArgs<T> + ?Sized,
        As::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as_with::<T, As>(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn ensure_reference(&self) -> Result<(), CellBuilderError> {
        if self.references.len() == MAX_REFS_COUNT {
            return Err(Error::custom("too many references"));
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn store_reference_as<T, As>(
        &mut self,
        value: T,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        self.ensure_reference()?;
        let mut builder = Self::new();
        builder.store_as::<T, As>(value)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }

    #[inline]
    pub(crate) fn store_reference_as_with<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAsWithArgs<T> + ?Sized,
    {
        self.ensure_reference()?;
        let mut builder = Self::new();
        builder.store_as_with::<T, As>(value, args)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }

    #[inline]
    #[must_use]
    pub fn into_cell(self) -> Cell {
        Cell {
            data: self.data.into_inner(),
            references: self.references,
        }
    }
}

impl BitWriter for CellBuilder {
    type Error = <CellBitWriter as BitWriter>::Error;

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.data.write_bit(bit)?;
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.data.write_bitslice(bits)
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.data.repeat_bit(n, bit)
    }
}
