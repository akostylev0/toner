use std::{rc::Rc, sync::Arc};

use crate::{either::Either, r#as::AsWrap, OrdinaryCell, ResultExt};

use super::{CellBuilder, CellBuilderError, CellSerialize};

/// Adapter to **ser**ialize `T`.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For dynamic arguments, see
/// [`CellSerializeAsWithArgs`](super::args::as::CellSerializeAsWithArgs).
pub trait CellSerializeAs<T: ?Sized, C = OrdinaryCell> {
    /// Store given value using an adapter
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>>;
}

impl<'a, T, As, C> CellSerializeAs<&'a T, C> for &'a As
where
    As: CellSerializeAs<T, C> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &&T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        AsWrap::<&T, As>::new(source).store(builder)
    }
}

impl<'a, T, As, C> CellSerializeAs<&'a mut T, C> for &'a mut As
where
    As: CellSerializeAs<T, C> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &&mut T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        AsWrap::<&T, As>::new(source).store(builder)
    }
}

impl<T, As, C> CellSerializeAs<[T], C> for [As]
where
    As: CellSerializeAs<T, C>,
{
    #[inline]
    fn store_as(source: &[T], builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        for (i, v) in source.iter().enumerate() {
            builder
                .store_as::<&T, &As>(v)
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(())
    }
}

impl<T, As, const N: usize, C> CellSerializeAs<[T; N], C> for [As; N]
where
    As: CellSerializeAs<T, C>,
{
    #[inline]
    fn store_as(source: &[T; N], builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        builder.store_as::<&[T], &[As]>(source)?;
        Ok(())
    }
}

macro_rules! impl_cell_serialize_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+, C> CellSerializeAs<($($t,)+), C> for ($($a,)+)
        where $(
            $a: CellSerializeAs<$t, C>,
        )+
        {
            #[inline]
            fn store_as(source: &($($t,)+), builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
                builder$(
                    .store_as::<&$t, &$a>(&source.$n)?)+;
                Ok(())
            }
        }
    };
}
impl_cell_serialize_as_for_tuple!(0:T0 as As0);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, As, C> CellSerializeAs<Box<T>, C> for Box<As>
where
    As: CellSerializeAs<T, C> + ?Sized,
{
    #[inline]
    fn store_as(source: &Box<T>, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        AsWrap::<&T, As>::new(source).store(builder)
    }
}

impl<T, As, C> CellSerializeAs<Rc<T>, C> for Rc<As>
where
    As: CellSerializeAs<T, C> + ?Sized,
{
    #[inline]
    fn store_as(source: &Rc<T>, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        AsWrap::<&T, As>::new(source).store(builder)
    }
}

impl<T, As, C> CellSerializeAs<Arc<T>, C> for Arc<As>
where
    As: CellSerializeAs<T, C> + ?Sized,
{
    #[inline]
    fn store_as(source: &Arc<T>, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        AsWrap::<&T, As>::new(source).store(builder)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<Left, Right, AsLeft, AsRight, C> CellSerializeAs<Either<Left, Right>, C> for Either<AsLeft, AsRight>
where
    AsLeft: CellSerializeAs<Left, C>,
    AsRight: CellSerializeAs<Right, C>,
{
    #[inline]
    fn store_as(
        source: &Either<Left, Right>,
        builder: &mut CellBuilder<C>,
    ) -> Result<(), CellBuilderError<C>> {
        source
            .as_ref()
            .map_either(AsWrap::<&Left, AsLeft>::new, AsWrap::<&Right, AsRight>::new)
            .store(builder)
    }
}

impl<T, As, C> CellSerializeAs<Option<T>, C> for Either<(), As>
where
    As: CellSerializeAs<T, C>,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        match source.as_ref() {
            None => Either::Left(()),
            Some(v) => Either::Right(AsWrap::<&T, As>::new(v)),
        }
        .store(builder)
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T, As, C> CellSerializeAs<Option<T>, C> for Option<As>
where
    As: CellSerializeAs<T, C>,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        source.as_ref().map(AsWrap::<_, As>::new).store(builder)
    }
}

pub trait CellSerializeWrapAsExt {
    #[inline]
    fn wrap_as<As>(&self) -> AsWrap<&'_ Self, As>
    where
        As: CellSerializeAs<Self> + ?Sized,
    {
        AsWrap::new(self)
    }
}
impl<T> CellSerializeWrapAsExt for T {}
