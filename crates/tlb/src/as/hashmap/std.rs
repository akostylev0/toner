use super::aug::{Hashmap, HashmapAugE, HashmapE, Key};
use super::hm_label::HmLabel;
use crate::{
    Context,
    r#as::{ParseFully, Ref},
    bits::{
        bitvec::{order::Msb0, vec::BitVec},
        de::BitReaderExt,
    },
    de::{CellDeserializeAs, CellParser, CellParserError},
};
use tlbits::adapters::Owned;
use tlbits::de::BitUnpack;

struct HashmapParserIter<'de, K, T, As>
where
    K: BitUnpack<'de>,
    As: CellDeserializeAs<'de, T>,
{
    stack: Vec<(u32, Key, CellParser<'de>)>,
    value_args: As::Args,
    key_args: K::Args,
    _phantom: std::marker::PhantomData<fn() -> (K, T)>,
}

impl<'de, K, T, As> HashmapParserIter<'de, K, T, As>
where
    K: BitUnpack<'de>,
    K::Args: Clone,
    As: CellDeserializeAs<'de, T>,
    As::Args: Clone,
{
    fn new(
        parser: &mut CellParser<'de>,
        n: u32,
        value_args: As::Args,
        key_args: K::Args,
    ) -> Result<Self, CellParserError<'de>> {
        let mut iter = Self {
            stack: Vec::new(),
            value_args,
            key_args,
            _phantom: std::marker::PhantomData,
        };
        iter.descend(parser, n, Key::default())?;
        Ok(iter)
    }

    fn descend(
        &mut self,
        parser: &mut CellParser<'de>,
        n: u32,
        mut prefix: Key,
    ) -> Result<Option<(K, T)>, CellParserError<'de>> {
        // label:(HmLabel ~l n)
        let next_prefix: BitVec<u8, Msb0> = parser.unpack_as::<_, HmLabel>(n).context("label")?;
        // {n = (~m) + l}
        let m = n - next_prefix.len() as u32;

        prefix.extend_from_bitslice(&next_prefix);

        match m {
            // bt_leaf$0
            0 => {
                let value = parser.parse_as::<_, As>(self.value_args.clone())?;
                let mut key_parser = Owned::new(prefix);
                let key = key_parser.unpack(self.key_args.clone())?;
                Ok(Some((key, value)))
            }
            // bt_fork$1
            1.. => {
                self.stack.extend(
                    parser
                        .parse_as::<_, [Ref; 2]>(())?
                        .into_iter()
                        .enumerate()
                        // HashmapNode (n + 1)
                        .map(|(next_prefix, parser)| {
                            let mut prefix = prefix.clone();
                            prefix.push(next_prefix != 0);
                            (m - 1, prefix, parser)
                        })
                        // inverse ordering
                        .rev(),
                );
                Ok(None)
            }
        }
    }
}

impl<'de, K, T, As> Iterator for HashmapParserIter<'de, K, T, As>
where
    K: BitUnpack<'de>,
    K::Args: Clone,
    As: CellDeserializeAs<'de, T>,
    As::Args: Clone,
{
    type Item = Result<(K, T), CellParserError<'de>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (n, prefix, mut parser) = self.stack.pop()?;
            match self.descend(&mut parser, n, prefix) {
                Ok(Some(entry)) => return Some(Ok(entry)),
                Ok(None) => continue,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl<'de, K, T, As, C> CellDeserializeAs<'de, C> for Hashmap<As>
where
    K: BitUnpack<'de>,
    K::Args: Clone,
    C: IntoIterator<Item = (K, T)> + FromIterator<(K, T)>,
    As: CellDeserializeAs<'de, T>,
    As::Args: Clone,
{
    /// (n, As::Args, K::Args)
    type Args = (u32, As::Args, K::Args);

    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
        (n, value_args, key_args): Self::Args,
    ) -> Result<C, CellParserError<'de>> {
        HashmapParserIter::<K, T, As>::new(parser, n, value_args, key_args)?
            .collect::<Result<C, _>>()
    }
}

impl<'de, K, T, As, C> CellDeserializeAs<'de, C> for HashmapE<As>
where
    C: IntoIterator<Item = (K, T)> + FromIterator<(K, T)>,
    K: BitUnpack<'de>,
    As: CellDeserializeAs<'de, T>,
    K::Args: Clone,
    As::Args: Clone,
{
    // (n, As::Args, K::Args)
    type Args = (u32, As::Args, K::Args);

    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
        (n, node_args, key_args): Self::Args,
    ) -> Result<C, CellParserError<'de>> {
        Ok(match parser.unpack(())? {
            // hme_empty$0
            false => std::iter::empty().collect(),
            // hme_root$1
            true => parser
                // root:^(Hashmap n X)
                .parse_as::<_, Ref<ParseFully<Hashmap<As, ()>>>>((n, node_args, key_args))?,
        })
    }
}

impl<'de, K, T, As, C> CellDeserializeAs<'de, C> for HashmapAugE<As>
where
    C: IntoIterator<Item = (K, T)> + FromIterator<(K, T)>,
    K: BitUnpack<'de>,
    As: CellDeserializeAs<'de, T>,
    K::Args: Clone,
    As::Args: Clone,
{
    // (n, As::Args, K::Args)
    type Args = (u32, As::Args, K::Args);

    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
        (n, node_args, key_args): Self::Args,
    ) -> Result<C, CellParserError<'de>> {
        let c: C = parser.parse_as::<_, HashmapE<As>>((n, node_args, key_args))?;
        // extra:Y = ()
        parser.parse::<()>(())?;
        Ok(c)
    }
}
