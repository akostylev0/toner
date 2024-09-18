use core::marker::PhantomData;

use tlbits::{either::Either, r#as::args::NoArgs, ser::BitWriter};

use crate::{
    de::{
        args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
    },
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
    Cell, ResultExt,
};

use super::Same;

/// Adapter to **de**/**ser**ialize value from/into reference to the child cell.
pub struct Ref<As: ?Sized = Same>(PhantomData<As>);

impl<C, T, As> CellSerializeAs<C, T> for Ref<As>
where
    As: CellSerializeAs<C, T> + ?Sized,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        builder.store_reference_as::<&T, &As>(source).context("^")?;
        Ok(())
    }
}

impl<C, T, As> CellSerializeAsWithArgs<C, T> for Ref<As>
where
    As: CellSerializeAsWithArgs<C, T> + ?Sized,
{
    type Args = As::Args;

    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        builder
            .store_reference_as_with::<&T, &As>(source, args)
            .context("^")?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Ref<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        parser.parse_reference_as::<T, As>().context("^")
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for Ref<As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        parser.parse_reference_as_with::<T, As>(args).context("^")
    }
}

/// ```tlb
/// {X:Type} Either X ^X = EitherInlineOrRef X
/// ```
pub struct EitherInlineOrRef<As: ?Sized = Same>(PhantomData<As>);

impl<C, T, As> CellSerializeAs<C, T> for EitherInlineOrRef<As>
where
    As: CellSerializeAs<C, T>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        EitherInlineOrRef::<NoArgs<(), As>>::store_as_with(source, builder, ())
    }
}

impl<C, T, As> CellSerializeAsWithArgs<C, T> for EitherInlineOrRef<As>
where
    As: CellSerializeAsWithArgs<C, T>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        let mut b = Cell::builder();
        As::store_as_with(source, &mut b, args)?;
        let cell = b.into_cell();
        builder.store_as::<_, Either<Same, Ref>>(
            if cell.len() <= builder.capacity_left() {
                Either::Left
            } else {
                Either::Right
            }(cell),
        )?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for EitherInlineOrRef<As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        EitherInlineOrRef::<NoArgs<(), As>>::parse_as_with(parser, ())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for EitherInlineOrRef<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        Either::<As, Ref<As>>::parse_as_with(parser, args).map(Either::into_inner)
    }
}
