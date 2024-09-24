use std::marker::PhantomData;
use std::sync::Arc;

use crate::{
    bits::{
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        ser::{BitWriter, LimitWriter},
    },
    r#as::Ref,
    Cell, CellBehavior, Error, OrdinaryCell, ResultExt,
};

use super::{
    args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
    r#as::CellSerializeAs,
    CellSerialize,
};

type CellBitWriter = LimitWriter<BitVec<u8, Msb0>>;

/// [`Error`] for [`CellBuilder`]
pub type CellBuilderError<C = OrdinaryCell> = <CellBuilder<C> as BitWriter>::Error;

/// Cell builder created with [`Cell::builder()`].
///
/// [`CellBuilder`] can then be converted to constructed [`Cell`] by using
/// [`.into_cell()`](CellBuilder::into_cell).
pub struct CellBuilder<C = OrdinaryCell> {
    r#type: PhantomData<C>,
    data: CellBitWriter,
    references: Vec<Arc<Cell>>,
}

const MAX_BITS_LEN: usize = 1023;
const MAX_REFS_COUNT: usize = 4;

impl<C> CellBuilder<C> {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            r#type: PhantomData,
            data: LimitWriter::new(BitVec::EMPTY, MAX_BITS_LEN),
            references: Vec::new(),
        }
    }

    /// Store the value using its [`CellSerialize`] implementation
    #[inline]
    pub fn store<T>(&mut self, value: T) -> Result<&mut Self, CellBuilderError<C>>
    where
        T: CellSerialize<C>,
    {
        value.store(self)?;
        Ok(self)
    }

    /// Store the value with args using its [`CellSerializeWithArgs`]
    /// implementation
    #[inline]
    pub fn store_with<T>(
        &mut self,
        value: T,
        args: T::Args,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        T: CellSerializeWithArgs<C>,
    {
        value.store_with(self, args)?;
        Ok(self)
    }

    /// Store all values from given iterator using [`CellSerialize`]
    /// implementation of its item type.
    #[inline]
    pub fn store_many<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        T: CellSerialize<C>,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store all values from given iterator with args using
    /// [`CellSerializeWithArgs`] implementation of its item type.
    #[inline]
    pub fn store_many_with<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: T::Args,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        T: CellSerializeWithArgs<C>,
        T::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_with(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store given value using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_as<T, As>(&mut self, value: T) -> Result<&mut Self, CellBuilderError<C>>
    where
        As: CellSerializeAs<T, C> + ?Sized,
    {
        As::store_as(&value, self)?;
        Ok(self)
    }

    /// Store given value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_as_with<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        As: CellSerializeAsWithArgs<T, C> + ?Sized,
    {
        As::store_as_with(&value, self, args)?;
        Ok(self)
    }

    /// Store all values from iterator using an adapter.  s
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_many_as<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T, C> + ?Sized,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as::<T, As>(v)
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store all values from iterator with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_many_as_with<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        As: CellSerializeAsWithArgs<T, C> + ?Sized,
        As::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as_with::<T, As>(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn ensure_reference(&self) -> Result<(), CellBuilderError<C>> {
        if self.references.len() == MAX_REFS_COUNT {
            return Err(Error::custom("too many references"));
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn store_reference_as<T, As, I>(
        &mut self,
        value: T,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        As: CellSerializeAs<T, I> + ?Sized,
        I: CellBehavior,
    {
        self.ensure_reference()?;
        let mut builder = CellBuilder::<I>::new();
        builder.store_as::<T, As>(value)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }

    #[inline]
    pub(crate) fn store_reference_as_with<T, As, I>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError<C>>
    where
        As: CellSerializeAsWithArgs<T, I> + ?Sized,
        I: CellBehavior,
    {
        self.ensure_reference()?;
        let mut builder = CellBuilder::<I>::new();
        builder.store_as_with::<T, As>(value, args)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }
}

impl<C> CellBuilder<C>
where
    C: CellBehavior,
{
    pub fn into_cell(self) -> Cell {
        let inner = C::new(self.data.into_inner(), self.references);
        inner.into()
    }
}

impl<C> BitWriter for CellBuilder<C> {
    type Error = <CellBitWriter as BitWriter>::Error;

    #[inline]
    fn capacity_left(&self) -> usize {
        self.data.capacity_left()
    }

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

impl CellSerialize for CellBuilder {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.write_bitslice(&self.data)?;
        builder.store_many_as::<_, Ref>(&self.references)?;
        Ok(())
    }
}
