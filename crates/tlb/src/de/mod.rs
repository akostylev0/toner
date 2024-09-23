//! **De**serialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
pub mod args;
pub mod r#as;
mod parser;

pub use self::parser::*;

use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use crate::{
    bits::de::BitReaderExt,
    either::Either,
    r#as::{FromInto, Same},
    Cell, CellBehavior, OrdinaryCell, ResultExt,
};

/// A type that can be **de**serialized from [`OrdinaryCellParser`].
pub trait CellDeserialize<'de, C = OrdinaryCell>: Sized {
    /// Parse value
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>>;
}

/// Owned version of [`CellDeserialize`]
pub trait CellDeserializeOwned<C = OrdinaryCell>: for<'de> CellDeserialize<'de, C> {}
impl<T, C> CellDeserializeOwned<C> for T where T: for<'de> CellDeserialize<'de, C> {}

impl<'de, C> CellDeserialize<'de, C> for () {
    #[inline]
    fn parse(_parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        Ok(())
    }
}

impl<'de, C, T, const N: usize> CellDeserialize<'de, C> for [T; N]
where
    T: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(T::parse(parser)?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_cell_deserialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, C, $($t),+> CellDeserialize<'de, C> for ($($t,)+)
        where $(
            $t: CellDeserialize<'de, C>,
        )+
        {
            #[inline]
            fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>>
            {
                Ok(($(
                    $t::parse(parser).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_for_tuple!(0:T0);
impl_cell_deserialize_for_tuple!(0:T0,1:T1);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<'de, T, C> CellDeserialize<'de, C> for Box<T>
where
    T: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        parser.parse_as::<_, FromInto<T>>()
    }
}

impl<'de, T, C> CellDeserialize<'de, C> for Rc<T>
where
    T: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        parser.parse_as::<_, FromInto<T>>()
    }
}

impl<'de, T, C> CellDeserialize<'de, C> for Arc<T>
where
    T: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        parser.parse_as::<_, FromInto<T>>()
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right, C> CellDeserialize<'de, C> for Either<Left, Right>
where
    Left: CellDeserialize<'de, C>,
    Right: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        match parser.unpack().context("tag")? {
            false => parser.parse().map(Either::Left).context("left"),
            true => parser.parse().map(Either::Right).context("right"),
        }
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T, C> CellDeserialize<'de, C> for Option<T>
where
    T: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        parser.parse_as::<_, Either<(), Same>>()
    }
}

impl<'de, C> CellDeserialize<'de, C> for Cell
where
    C: CellBehavior + CellDeserialize<'de, C>,
    Cell: From<C>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        let cell = C::parse(parser)?;

        Ok(cell.into())
    }
}
