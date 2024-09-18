use tlb::{de::{CellDeserialize, CellParser, CellParserError}, either::Either, r#as::{Data, Ref}, ser::{CellBuilder, CellBuilderError, CellSerialize}, Cell, OrdinaryCell};

/// ```tlb
/// libref_hash$0 lib_hash:bits256 = LibRef;
/// libref_ref$1 library:^Cell = LibRef;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibRef<R = Cell> {
    Hash([u8; 32]),
    Ref(R),
}

impl<R> CellSerialize<OrdinaryCell> for LibRef<R>
where
    R: CellSerialize<OrdinaryCell>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder<OrdinaryCell>) -> Result<(), CellBuilderError<OrdinaryCell>> {
        builder.store_as::<_, Either<Data, Ref>>(match self {
            Self::Hash(hash) => Either::Left(hash),
            Self::Ref(library) => Either::Right(library),
        })?;
        Ok(())
    }
}

impl<'de, R> CellDeserialize<'de> for LibRef<R>
where
    R: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(match parser.parse_as::<_, Either<Data, Ref>>()? {
            Either::Left(hash) => Self::Hash(hash),
            Either::Right(library) => Self::Ref(library),
        })
    }
}
