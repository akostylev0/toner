pub mod r#as;

use std::{rc::Rc, sync::Arc};

use crate::{bits::ser::BitWriterExt, either::Either, r#as::Same, OrdinaryCell, ResultExt};

use super::{CellBuilder, CellBuilderError};

/// A type that can be **ser**ialized.  
/// In contrast with [`CellSerialize`](super::CellSerialize) it allows to pass
/// [`Args`](CellSerializeWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
pub trait CellSerializeWithArgs<C = OrdinaryCell> {
    type Args;

    /// Stores the value with args
    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>>;
}

impl<T, C> CellSerializeWithArgs<C> for &T
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        <T as CellSerializeWithArgs<C>>::store_with(self, builder, args)
    }
}

impl<T, C> CellSerializeWithArgs<C> for &mut T
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        <T as CellSerializeWithArgs<C>>::store_with(self, builder, args)
    }
}

impl<T, C> CellSerializeWithArgs<C> for Box<T>
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        <T as CellSerializeWithArgs<C>>::store_with(self, builder, args)
    }
}

impl<T, C> CellSerializeWithArgs<C> for Rc<T>
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        <T as CellSerializeWithArgs<C>>::store_with(self, builder, args)
    }
}

impl<T, C> CellSerializeWithArgs<C> for Arc<T>
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        <T as CellSerializeWithArgs<C>>::store_with(self, builder, args)
    }
}

impl<T, C> CellSerializeWithArgs<C> for [T]
where
    T: CellSerializeWithArgs<C>,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        builder.store_many_with(self, args)?;
        Ok(())
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<L, R, C> CellSerializeWithArgs<C> for Either<L, R>
where
    L: CellSerializeWithArgs<C>,
    R: CellSerializeWithArgs<C, Args = L::Args>,
{
    type Args = L::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        match self {
            Self::Left(l) => builder
                .pack(false)
                .context("tag")?
                .store_with(l, args)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true)
                .context("tag")?
                .store_with(r, args)
                .context("right")?,
        };
        Ok(())
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T, C> CellSerializeWithArgs<C> for Option<T>
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        builder.store_as_with::<_, Either<(), Same>>(self.as_ref(), args)?;
        Ok(())
    }
}
