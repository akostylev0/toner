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

pub use crate::bits::r#as::Same;
use crate::de::{CellParser, CellParserError};

impl<T, C> CellSerializeAs<T, C> for Same
where
    T: CellSerialize<C>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        source.store(builder)
    }
}

impl<T, C> CellSerializeAsWithArgs<T, C> for Same
where
    T: CellSerializeWithArgs<C>,
{
    type Args = T::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        T::store_with(source, builder, args)
    }
}

impl<'de, T, C> CellDeserializeAs<'de, T, C> for Same
where
    T: CellDeserialize<'de, C>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        T::parse(parser)
    }
}

impl<'de, T, C> CellDeserializeAsWithArgs<'de, T, C> for Same
where
    T: CellDeserializeWithArgs<'de, C>,
{
    type Args = T::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>> {
        T::parse_with(parser, args)
    }
}
