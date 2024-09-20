use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use crate::{either::Either, r#as::{AsWrap, NoArgs}, OrdinaryCell, ResultExt};
use crate::de::{CellParser, CellParserError};
use super::{
    super::{OrdinaryCellParser, OrdinaryCellParserError},
    CellDeserializeWithArgs,
};

/// Adaper to **de**serialize `T` with args.
/// See [`as`](crate::as) module-level documentation for more.
///
/// For version without arguments, see
/// [`CellDeserializeAs`](super::super::as::CellDeserializeAs).
pub trait CellDeserializeAsWithArgs<'de, T, C = OrdinaryCell> {
    type Args;

    /// Parse value with args using an adapter
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>>;
}

/// Owned version of [`CellDeserializeAsWithArgs`]
pub trait CellDeserializeAsWithArgsOwned<T, C = OrdinaryCell>: for<'de> CellDeserializeAsWithArgs<'de, T, C> {}
impl<T, As, C> CellDeserializeAsWithArgsOwned<As, C> for T where
    T: for<'de> CellDeserializeAsWithArgs<'de, As, C> + ?Sized
{
}

impl<'de, T, As, const N: usize> CellDeserializeAsWithArgs<'de, [T; N]> for [As; N]
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut OrdinaryCellParser<'de>,
        args: Self::Args,
    ) -> Result<[T; N], OrdinaryCellParserError<'de>> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(parser.parse_as_with::<T, As>(args.clone())?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

impl<'de, 'a: 'de, T, As, C> CellDeserializeAsWithArgs<'de, Vec<T>, C> for Vec<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C>,
    As::Args: Clone + 'a,
{
    type Args = (usize, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        (len, args): Self::Args,
    ) -> Result<Vec<T>, CellParserError<'de, C>> {
        parser.parse_iter_as_with::<_, As>(args).take(len).collect()
    }
}

macro_rules! impl_cell_deserialize_as_with_args_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<'de, C, $($t, $a),+> CellDeserializeAsWithArgs<'de, ($($t,)+), C> for ($($a,)+)
        where $(
            $a: CellDeserializeAsWithArgs<'de, $t, C>,
        )+
        {
            type Args = ($($a::Args,)+);

            #[inline]
            fn parse_as_with(parser: &mut CellParser<'de, C>, args: Self::Args) -> Result<($($t,)+), CellParserError<'de, C>>
            {
                Ok(($(
                    $a::parse_as_with(parser, args.$n)
                        .context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, Box<T>, C> for Box<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Box<T>, CellParserError<'de, C>> {
        AsWrap::<T, As>::parse_with(parser, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, Rc<T>, C> for Rc<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Rc<T>, CellParserError<'de, C>> {
        AsWrap::<T, As>::parse_with(parser, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, Arc<T>, C> for Arc<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Arc<T>, CellParserError<'de, C>> {
        AsWrap::<T, As>::parse_with(parser, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right, AsLeft, AsRight, C> CellDeserializeAsWithArgs<'de, Either<Left, Right>, C>
    for Either<AsLeft, AsRight>
where
    AsLeft: CellDeserializeAsWithArgs<'de, Left, C>,
    AsRight: CellDeserializeAsWithArgs<'de, Right, C, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Either<Left, Right>, CellParserError<'de, C>> {
        Ok(
            Either::<AsWrap<Left, AsLeft>, AsWrap<Right, AsRight>>::parse_with(parser, args)?
                .map_either(AsWrap::into_inner, AsWrap::into_inner),
        )
    }
}

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, Option<T>, C> for Either<(), As>
where
    As: CellDeserializeAsWithArgs<'de, T, C>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Option<T>, CellParserError<'de, C>> {
        Ok(parser
            .parse_as_with::<Either<(), T>, Either<NoArgs<_>, As>>(args)?
            .right())
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, Option<T>, C> for Option<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Option<T>, CellParserError<'de, C>> {
        Ok(Option::<AsWrap<T, As>>::parse_with(parser, args)?.map(AsWrap::into_inner))
    }
}
