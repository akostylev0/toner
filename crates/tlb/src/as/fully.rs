use core::marker::PhantomData;

use crate::de::{
    args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
};

use super::Same;

/// Adapter to **de**serialize value and ensure that no more data and references
/// left.
pub struct ParseFully<As: ?Sized = Same>(PhantomData<As>);

impl<'de, T, As, C> CellDeserializeAs<'de, T, C> for ParseFully<As>
where
    As: CellDeserializeAs<'de, T, C> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        let v = parser.parse_as::<_, As>()?;
        parser.ensure_empty()?;
        Ok(v)
    }
}

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, T, C> for ParseFully<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>> {
        let v = parser.parse_as_with::<_, As>(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }
}
