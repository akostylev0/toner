use crate::{
    de::{args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs},
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
};

pub use crate::bits::r#as::args::{DefaultArgs, NoArgs};
use crate::de::{CellParser, CellParserError};

impl<T, As, Args, C> CellSerializeAsWithArgs<T, C> for NoArgs<Args, As>
where
    As: CellSerializeAs<T, C> + ?Sized,
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

impl<'de, T, As, Args, C> CellDeserializeAsWithArgs<'de, T, C> for NoArgs<Args, As>
where
    As: CellDeserializeAs<'de, T, C> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        _args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>> {
        As::parse_as(parser)
    }
}

impl<T, As> CellSerializeAs<T> for DefaultArgs<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Default,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::store_as_with(source, builder, <As::Args>::default())
    }
}

impl<'de, T, As, C> CellDeserializeAs<'de, T, C> for DefaultArgs<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C>,
    As::Args: Default,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        As::parse_as_with(parser, <As::Args>::default())
    }
}
