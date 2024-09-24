use core::marker::PhantomData;

use tlbits::{either::Either, r#as::args::NoArgs, ser::BitWriter};

use super::Same;
use crate::cell::CellBehavior;
use crate::de::{CellParser, CellParserError};
use crate::{de::{args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs}, ser::{
    args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
}, Cell, OrdinaryCell, ResultExt};

/// Adapter to **de**/**ser**ialize value from/into reference to the child cell.
pub struct Ref<As: ?Sized = Same, Cell :?Sized = OrdinaryCell>(PhantomData<As>, PhantomData<Cell>);

impl<T, As, C, I> CellSerializeAs<T, C> for Ref<As, I>
where
    As: CellSerializeAs<T, I> + ?Sized,
    I: CellBehavior
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        builder.store_reference_as::<&T, &As, I>(source).context("^")?;
        Ok(())
    }
}

impl<T, As, C, I> CellSerializeAsWithArgs<T, C> for Ref<As, I>
where
    As: CellSerializeAsWithArgs<T, I> + ?Sized,
    I: CellBehavior
{
    type Args = As::Args;

    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        builder
            .store_reference_as_with::<&T, &As, I>(source, args)
            .context("^")?;
        Ok(())
    }
}

impl<'de, T, As, C, I> CellDeserializeAs<'de, T, C> for Ref<As, I>
where
    As: CellDeserializeAs<'de, T, I> + ?Sized,
    I: CellBehavior + 'de,
    &'de Cell: TryInto<&'de I>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de, C>) -> Result<T, CellParserError<'de, C>> {
        parser.parse_reference_as::<T, As, I>().context("^")
    }
}

impl<'de, T, As, C, I> CellDeserializeAsWithArgs<'de, T, C> for Ref<As, I>
where
    As: CellDeserializeAsWithArgs<'de, T, I> + ?Sized,
    &'de Cell: TryInto<&'de I>,
    I: CellBehavior + 'de
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de, C>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de, C>> {
        parser
            .parse_reference_as_with::<T, As, I>(args)
            .context("^")
    }
}

/// ```tlb
/// {X:Type} Either X ^X = EitherInlineOrRef X
/// ```
pub struct EitherInlineOrRef<As: ?Sized = Same>(PhantomData<As>);

impl<T, As, C> CellSerializeAs<T, C> for EitherInlineOrRef<As>
where
    As: CellSerializeAs<T, C>,
    C: CellBehavior
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder<C>) -> Result<(), CellBuilderError<C>> {
        EitherInlineOrRef::<NoArgs<(), As>>::store_as_with(source, builder, ())
    }
}

impl<T, As, C> CellSerializeAsWithArgs<T, C> for EitherInlineOrRef<As>
where
    As: CellSerializeAsWithArgs<T, C>,
    C: CellBehavior
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder<C>,
        args: Self::Args,
    ) -> Result<(), CellBuilderError<C>> {
        let mut b = Cell::builder::<C>();
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
        Either::<As, Ref<As, C>>::parse_as_with(parser, args).map(Either::into_inner)
    }
}
