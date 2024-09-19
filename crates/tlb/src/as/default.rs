use crate::{
    de::r#as::CellDeserializeAs,
    ser::{r#as::CellSerializeAs, CellBuilder, CellBuilderError},
};

pub use crate::bits::r#as::DefaultOnNone;
use crate::de::{CellParser, CellParserError};

impl<T, As> CellSerializeAs<Option<T>> for DefaultOnNone<As>
where
    As: CellSerializeAs<T>,
    T: Default,
{
    fn store_as(source: &Option<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match source {
            Some(v) => builder.store_as::<_, &As>(v)?,
            None => builder.store_as::<_, As>(T::default())?,
        };
        Ok(())
    }
}

impl<'de, T, As, C> CellDeserializeAs<'de, T, C> for DefaultOnNone<As>
where
    T: Default,
    As: CellDeserializeAs<'de, T, C>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        parser
            .parse_as::<_, Option<As>>()
            .map(Option::unwrap_or_default)
    }
}
