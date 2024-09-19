//! **De**/**ser**ialization helpers for
//! [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language).
//!
//! This approach is heavily inspired by
//! [serde_with](https://docs.rs/serde_with/latest/serde_with).
//! Please, read their docs for more usage examples.
mod args;
mod data;
mod default;
mod from_into;
mod fully;
mod reference;
mod same;

pub use self::{args::*, data::*, default::*, from_into::*, fully::*, reference::*, same::*};

use crate::{
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellDeserialize,
    },
    ser::{
        args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
        r#as::CellSerializeAs,
        CellBuilder, CellBuilderError, CellSerialize,
    },
};

use crate::de::{CellParser, CellParserError};
pub use tlbits::r#as::AsWrap;

impl<'a, T, As> CellSerialize for AsWrap<&'a T, As>
where
    T: ?Sized,
    As: ?Sized,
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::store_as(self.into_inner(), builder)
    }
}

impl<'a, T, As> CellSerializeWithArgs for AsWrap<&'a T, As>
where
    T: ?Sized,
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::store_as_with(self.into_inner(), builder, args)
    }
}

impl<'de, T, As, C> CellDeserialize<'de, C> for AsWrap<T, As>
where
    As: CellDeserializeAs<'de, T, C> + ?Sized,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de, C>) -> Result<Self, CellParserError<'de, C>> {
        As::parse_as(parser).map(Self::new)
    }
}

impl<'de, T, As, C> CellDeserializeWithArgs<'de, C> for AsWrap<T, As>
where
    As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
{
    type Args = As::Args;

    fn parse_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de, C>> {
        As::parse_as_with(parser, args).map(Self::new)
    }
}
