use core::marker::PhantomData;

use tlbits::{either::Either, r#as::args::NoArgs, ser::BitWriter};

use super::Same;
use crate::cell::CellBehavior;
use crate::de::{CellParser, CellParserError};
use crate::{
    de::{args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs},
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
    Cell, ResultExt,
};

/// Adapter to **de**/**ser**ialize value from/into reference to the child cell.
pub struct Ref<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Ref<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_reference_as::<&T, &As>(source).context("^")?;
        Ok(())
    }
}

impl<T, As> CellSerializeAsWithArgs<T> for Ref<As>
where
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            .store_reference_as_with::<&T, &As>(source, args)
            .context("^")?;
        Ok(())
    }
}

impl<'de, T, As, C> CellDeserializeAs<'de, T, C> for Ref<As>
where
    As: CellDeserializeAs<'de, T, C> + ?Sized,
    C: CellBehavior + 'de,
    &'de Cell: TryInto<&'de C>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        parser.parse_reference_as::<T, As, C>().context("^")
    }
}

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, T, C> for Ref<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C> + ?Sized,
    &'de Cell: TryInto<&'de C>,
    C: CellBehavior + 'de
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>> {
        parser
            .parse_reference_as_with::<T, As, C>(args)
            .context("^")
    }
}

/// ```tlb
/// {X:Type} Either X ^X = EitherInlineOrRef X
/// ```
pub struct EitherInlineOrRef<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for EitherInlineOrRef<As>
where
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        EitherInlineOrRef::<NoArgs<(), As>>::store_as_with(source, builder, ())
    }
}

impl<T, As> CellSerializeAsWithArgs<T> for EitherInlineOrRef<As>
where
    As: CellSerializeAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        let mut b = Cell::builder();
        As::store_as_with(source, &mut b, args)?;
        let cell = b.into_cell();
        builder.store_as::<_, Either<Same, Ref>>(if cell.len() <= builder.capacity_left() {
            Either::Left
        } else {
            Either::Right
        }(cell))?;
        Ok(())
    }
}

impl<'de, T, As, C> CellDeserializeAs<'de, T, C> for EitherInlineOrRef<As>
where
    As: CellDeserializeAs<'de, T, C>,
    C: CellBehavior + 'de,
    &'de Cell: TryInto<&'de C>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        EitherInlineOrRef::<NoArgs<(), As>>::parse_as_with(parser, ())
    }
}

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, T, C> for EitherInlineOrRef<As>
where
    As: CellDeserializeAsWithArgs<'de, T, C>,
    C: CellBehavior + 'de,
    &'de Cell: TryInto<&'de C>
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>> {
        Either::<As, Ref<As>>::parse_as_with(parser, args).map(Either::into_inner)
    }
}
