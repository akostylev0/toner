use crate::{
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellDeserialize, OrdinaryCellParser, OrdinaryCellParserError,
    },
    ser::{
        args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
        r#as::CellSerializeAs,
        CellBuilder, CellBuilderError, CellSerialize,
    },
};

pub use crate::bits::r#as::Same;
use crate::de::{CellParser, CellParserError};

impl<T> CellSerializeAs<T> for Same
where
    T: CellSerialize,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source.store(builder)
    }
}

impl<T> CellSerializeAsWithArgs<T> for Same
where
    T: CellSerializeWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
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

impl<'de, T> CellDeserializeAsWithArgs<'de, T> for Same
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut OrdinaryCellParser<'de>,
        args: Self::Args,
    ) -> Result<T, OrdinaryCellParserError<'de>> {
        T::parse_with(parser, args)
    }
}
