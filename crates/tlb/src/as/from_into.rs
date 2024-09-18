use core::fmt::Display;

use crate::{
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellDeserialize, CellParser, CellParserError,
    },
    ser::{
        args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
        r#as::CellSerializeAs,
        CellBuilder, CellBuilderError, CellSerialize,
    },
    Error,
};

pub use crate::bits::r#as::{FromInto, FromIntoRef, TryFromInto, TryFromIntoRef};

impl<C, T, As> CellSerializeAs<C, T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: CellSerialize<C>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        source.clone().into().store(builder)
    }
}

impl<C, T, As> CellSerializeAsWithArgs<C, T> for FromInto<As>
where
    T: Into<As> + Clone,
    As: CellSerializeWithArgs<C>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        source.clone().into().store_with(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for FromInto<As>
where
    As: Into<T> + CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser).map(Into::into)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for FromInto<As>
where
    As: Into<T> + CellDeserializeWithArgs<'de>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args).map(Into::into)
    }
}

impl<C, T, As> CellSerializeAs<C, T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: CellSerialize<C>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        source.into().store(builder)
    }
}

impl<C, T, As> CellSerializeAsWithArgs<C, T> for FromIntoRef<As>
where
    for<'a> &'a T: Into<As>,
    As: CellSerializeWithArgs<C>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        source.into().store_with(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + CellDeserialize<'de>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser).map(Into::into)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for FromIntoRef<As>
where
    As: Into<T> + CellDeserializeWithArgs<'de>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args).map(Into::into)
    }
}

impl<C, T, As> CellSerializeAs<C, T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: CellSerialize<C>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store(builder)
    }
}

impl<C, T, As> CellSerializeAsWithArgs<C, T> for TryFromInto<As>
where
    T: TryInto<As> + Clone,
    <T as TryInto<As>>::Error: Display,
    As: CellSerializeWithArgs<C>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store_with(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + CellDeserialize<'de>,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser)?.try_into().map_err(Error::custom)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for TryFromInto<As>
where
    As: TryInto<T> + CellDeserializeWithArgs<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args)?
            .try_into()
            .map_err(Error::custom)
    }
}

impl<C, T, As> CellSerializeAs<C, T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As>,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: CellSerialize<C>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        source.try_into().map_err(Error::custom)?.store(builder)
    }
}

impl<C, T, As> CellSerializeAsWithArgs<C, T> for TryFromIntoRef<As>
where
    for<'a> &'a T: TryInto<As> + Clone,
    for<'a> <&'a T as TryInto<As>>::Error: Display,
    As: CellSerializeWithArgs<C>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        source
            .clone()
            .try_into()
            .map_err(Error::custom)?
            .store_with(builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + CellDeserialize<'de>,
    <As as TryInto<T>>::Error: Display,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse(parser)?.try_into().map_err(Error::custom)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for TryFromIntoRef<As>
where
    As: TryInto<T> + CellDeserializeWithArgs<'de>,
    <As as TryInto<T>>::Error: Display,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_with(parser, args)?
            .try_into()
            .map_err(Error::custom)
    }
}
