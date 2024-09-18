use crate::{
    de::{
        args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
    },
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
};

pub use crate::bits::r#as::args::{DefaultArgs, NoArgs};

impl<C, T, As, Args> CellSerializeAsWithArgs<C, T> for NoArgs<Args, As>
where
    As: CellSerializeAs<C, T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        _args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        As::store_as(source, builder)
    }
}

impl<'de, T, As, Args> CellDeserializeAsWithArgs<'de, T> for NoArgs<Args, As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        _args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_as(parser)
    }
}

impl<C, T, As> CellSerializeAs<C, T> for DefaultArgs<As>
where
    As: CellSerializeAsWithArgs<C, T>,
    As::Args: Default,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        As::store_as_with(source, builder, <As::Args>::default())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for DefaultArgs<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Default,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::parse_as_with(parser, <As::Args>::default())
    }
}
