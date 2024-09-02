use crate::{
    de::{r#as::CellDeserializeAs, CellParser, CellParserError},
    ser::{r#as::CellSerializeAs, OrdinaryCellBuilder, CellBuilderError},
};

pub use crate::bits::r#as::DefaultOnNone;

impl<T, As> CellSerializeAs<Option<T>> for DefaultOnNone<As>
where
    As: CellSerializeAs<T>,
    T: Default,
{
    fn store_as(source: &Option<T>, builder: &mut OrdinaryCellBuilder) -> Result<(), CellBuilderError> {
        match source {
            Some(v) => builder.store_as::<_, &As>(v)?,
            None => builder.store_as::<_, As>(T::default())?,
        };
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for DefaultOnNone<As>
where
    T: Default,
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        parser
            .parse_as::<_, Option<As>>()
            .map(Option::unwrap_or_default)
    }
}
