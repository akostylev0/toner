use core::marker::PhantomData;

use tlb::{
    bits::{
        bitvec::{order::Msb0, vec::BitVec},
        de::BitReaderExt,
        ser::BitWriterExt,
    },
    de::{args::r#as::CellDeserializeAsWithArgs, CellParser, CellParserError},
    r#as::{ParseFully, Ref, Same},
    ser::{args::r#as::CellSerializeAsWithArgs, CellBuilder, CellBuilderError},
    Error, ResultExt,
};

use super::{aug::AugNode, hm_label::HmLabel, Hashmap, HashmapE, Node};

/// ```tlb
/// phme_empty$0 {n:#} {X:Type} = PfxHashmapE n X;
/// phme_root$1 {n:#} {X:Type} root:^(PfxHashmap n X) = PfxHashmapE n X;
/// ```
pub struct PfxHashmapE<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAsWithArgs<HashmapE<T>> for PfxHashmapE<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn store_as_with(
        source: &HashmapE<T>,
        builder: &mut tlb::ser::CellBuilder,
        args: Self::Args,
    ) -> Result<(), tlb::ser::CellBuilderError> {
        match source {
            HashmapE::Empty => builder
                // phme_empty$0
                .pack(false)?,
            HashmapE::Root(root) => builder
                // phme_root$1
                .pack(true)?
                // root:^(PfxHashmap n X)
                .store_as_with::<_, Ref<&PfxHashmap<As>>>(root, args)?,
        };
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, HashmapE<T>> for PfxHashmapE<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut tlb::de::CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<HashmapE<T>, tlb::de::CellParserError<'de>> {
        Ok(match parser.unpack()? {
            // hme_empty$0
            false => HashmapE::Empty,
            // hme_root$1
            true => parser
                // root:^(Hashmap n X)
                .parse_as_with::<_, Ref<ParseFully<PfxHashmap<As>>>>((n, args))
                .map(HashmapE::Root)?,
        })
    }
}

/// ```tlb
/// phm_edge#_ {n:#} {X:Type} {l:#} {m:#} label:(HmLabel ~l n)
/// {n = (~m) + l} node:(PfxHashmapNode m X) = PfxHashmap n X;
/// ```
pub struct PfxHashmap<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAsWithArgs<Hashmap<T>> for PfxHashmap<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    fn store_as_with(
        source: &Hashmap<T>,
        builder: &mut CellBuilder,
        (n, args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            // label:(HmLabel ~l n)
            .pack_as_with::<_, &HmLabel>(source.prefix.as_bitslice(), n)
            .context("label")?
            // node:(PfxHashmapNode m X)
            .store_as_with::<_, &PfxHashmapNode<As>>(
                &source.node,
                (
                    // {n = (~m) + l}
                    n - source.prefix.len() as u32,
                    args,
                ),
            )
            .context("node")?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Hashmap<T>> for PfxHashmap<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<Hashmap<T>, CellParserError<'de>> {
        // label:(HmLabel ~l n)
        let prefix: BitVec<u8, Msb0> = parser.unpack_as_with::<_, HmLabel>(n).context("label")?;
        // {n = (~m) + l}
        let m = n - prefix.len() as u32;
        Ok(Hashmap {
            prefix,
            // node:(PfxHashmapNode m X)
            node: AugNode::new(
                parser
                    .parse_as_with::<_, PfxHashmapNode<As>>((m, args))
                    .context("node")?,
                (),
            ),
        })
    }
}

/// ```tlb
/// phmn_leaf$0 {n:#} {X:Type} value:X = PfxHashmapNode n X;
/// phmn_fork$1 {n:#} {X:Type} left:^(PfxHashmap n X)
/// right:^(PfxHashmap n X) = PfxHashmapNode (n + 1) X;
/// ```
pub struct PfxHashmapNode<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAsWithArgs<Node<T>> for PfxHashmapNode<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    fn store_as_with(
        source: &Node<T>,
        builder: &mut CellBuilder,
        (n, args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            Node::Leaf(value) => {
                builder
                    // phmn_leaf$0
                    .pack(false)?
                    // value:X
                    .store_as_with::<_, &As>(value, args)?
            }
            Node::Fork(fork) => {
                if n == 0 {
                    return Err(Error::custom("key is too long"));
                }
                builder
                    // phmn_fork$1
                    .pack(true)?
                    .store_as_with::<_, &[Box<Ref<PfxHashmap<As>>>; 2]>(fork, (n - 1, args))?
            }
        };
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Node<T>> for PfxHashmapNode<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n + 1, As::Args)
    type Args = (u32, As::Args);

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<Node<T>, CellParserError<'de>> {
        match parser.unpack()? {
            // phmn_leaf$0
            false => {
                // value:X
                parser.parse_as_with::<_, As>(args).map(Node::Leaf)
            }
            // phmn_fork$1
            true => {
                Ok(Node::Fork(
                    parser
                        // left:^(PfxHashmap n X) right:^(PfxHashmap n X)
                        .parse_as_with::<_, [Box<Ref<ParseFully<PfxHashmap<As>>>>; 2]>((
                            n - 1,
                            args,
                        ))?,
                ))
            }
        }
    }
}
