pub mod r#as;

use std::{mem::MaybeUninit, rc::Rc, sync::Arc};

use crate::{bits::de::BitReaderExt, either::Either, r#as::{FromInto, Same}, OrdinaryCell, ResultExt};

use super::{CellParser, CellParserError, OrdinaryCellParser, OrdinaryCellParserError};

/// A type that can be **de**serialized.  
/// In contrast with [`CellDeserialize`](super::CellDeserialize) it allows to
/// pass [`Args`](CellDeserializeWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
pub trait CellDeserializeWithArgs<'de, C = OrdinaryCell>: Sized {
    type Args;

    /// Parses the value with args
    fn parse_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de, C>>;
}

/// Owned version of [`CellDeserializeWithArgs`]
pub trait CellDeserializeWithArgsOwned<C>: for<'de> CellDeserializeWithArgs<'de, C> {}
impl<T, C> CellDeserializeWithArgsOwned<C> for T where T: for<'de> CellDeserializeWithArgs<'de, C> {}

impl<'de, T, const N: usize, C> CellDeserializeWithArgs<'de, C> for [T; N]
where
    T: CellDeserializeWithArgs<'de, C>,
    T::Args: Clone,
{
    type Args = T::Args;

    fn parse_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de, C>> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, a) in arr.iter_mut().enumerate() {
            a.write(T::parse_with(parser, args.clone()).with_context(|| format!("[{i}]"))?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_cell_deserialize_with_args_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+, C> CellDeserializeWithArgs<'de, C> for ($($t,)+)
        where $(
            $t: CellDeserializeWithArgs<'de, C>,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn parse_with(parser: &mut CellParser<'de, C>, args: Self::Args) -> Result<Self, CellParserError<'de, C>>
            {
                Ok(($(
                    $t::parse_with(parser, args.$n).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_with_args_for_tuple!(0:T0);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<'de, 'a: 'de, T, C> CellDeserializeWithArgs<'de, C> for Vec<T>
where
    T: CellDeserializeWithArgs<'de, C>,
    T::Args: Clone + 'a,
{
    type Args = (usize, T::Args);

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de, C>,
        (len, args): Self::Args,
    ) -> Result<Self, CellParserError<'de, C>> {
        parser.parse_iter_with(args).take(len).collect()
    }
}

impl<'de, T, C> CellDeserializeWithArgs<'de, C> for Box<T>
where
    T: CellDeserializeWithArgs<'de, C>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de, C>> {
        parser.parse_as_with::<_, FromInto<T>>(args)
    }
}

impl<'de, T> CellDeserializeWithArgs<'de> for Rc<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut OrdinaryCellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, OrdinaryCellParserError<'de>> {
        parser.parse_as_with::<_, FromInto<T>>(args)
    }
}

impl<'de, T> CellDeserializeWithArgs<'de> for Arc<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut OrdinaryCellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, OrdinaryCellParserError<'de>> {
        parser.parse_as_with::<_, FromInto<T>>(args)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right> CellDeserializeWithArgs<'de> for Either<Left, Right>
where
    Left: CellDeserializeWithArgs<'de>,
    Right: CellDeserializeWithArgs<'de, Args = Left::Args>,
{
    type Args = Left::Args;

    #[inline]
    fn parse_with(
        parser: &mut OrdinaryCellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, OrdinaryCellParserError<'de>> {
        match parser.unpack().context("tag")? {
            false => parser.parse_with(args).map(Either::Left).context("left"),
            true => parser.parse_with(args).map(Either::Right).context("right"),
        }
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T> CellDeserializeWithArgs<'de> for Option<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut OrdinaryCellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, OrdinaryCellParserError<'de>> {
        parser.parse_as_with::<_, Either<(), Same>>(args)
    }
}
