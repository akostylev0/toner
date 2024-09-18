//! **Ser**ialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
pub mod args;
pub mod r#as;
mod builder;

pub use self::builder::*;

use std::{rc::Rc, sync::Arc};

use impl_tools::autoimpl;

use crate::{bits::ser::BitWriterExt, either::Either, r#as::{Ref, Same}, Cell, CellMarker, ResultExt};

/// A type that can be **ser**ilalized into [`CellBuilder`].
#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait CellSerialize<C> {
    /// Store the value into [`CellBuilder`]
    fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>>;
}

impl<C> CellSerialize<C> for () {
    #[inline]
    fn store(&self, _builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        Ok(())
    }
}

impl<C, T> CellSerialize<C> for [T]
where
    T: CellSerialize<C>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        builder.store_many(self)?;
        Ok(())
    }
}

impl<C, T, const N: usize> CellSerialize<C> for [T; N]
where
    T: CellSerialize<C>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        self.as_slice().store(builder)
    }
}

macro_rules! impl_cell_serialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<C, $($t),+> CellSerialize<C> for ($($t,)+)
        where $(
            $t: CellSerialize<C>,
        )+
        {
            #[inline]
            fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>>
            {
                $(self.$n.store(builder).context(concat!(".", stringify!($n)))?;)+
                Ok(())
            }
        }
    };
}
impl_cell_serialize_for_tuple!(0:T0);
impl_cell_serialize_for_tuple!(0:T0,1:T1);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<C, L, R> CellSerialize<C> for Either<L, R>
where
    L: CellSerialize<C>,
    R: CellSerialize<C>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        match self {
            Self::Left(l) => builder
                .pack(false)
                .context("tag")?
                .store(l)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true)
                .context("tag")?
                .store(r)
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
impl<C, T> CellSerialize<C> for Option<T>
where
    T: CellSerialize<C>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        builder.store_as::<_, Either<(), Same>>(self.as_ref())?;
        Ok(())
    }
}

impl<C> CellSerialize<C> for Cell {
    #[inline]
    fn store(&self, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        builder
            .pack(self.as_bitslice())?
            .store_many_as::<_, Ref>(self.references())?;

        Ok(())
    }
}

pub trait CellSerializeExt<C>: CellSerialize<C> {
    #[inline]
    fn to_cell(&self) -> Result<Cell, CellBuilderError<C>> {
        let mut builder = Cell::builder();
        self.store(&mut builder)?;
        Ok(builder.into_cell())
    }
}
impl<C, T> CellSerializeExt<C> for T where T: CellSerialize<C> {}
