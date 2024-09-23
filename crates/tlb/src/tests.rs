use crate::{Cell, CellBehavior, de::r#as::CellDeserializeAsOwned, ser::{
    r#as::{CellSerializeAs, CellSerializeWrapAsExt},
    CellSerializeExt,
}};
use core::fmt::Debug;

#[track_caller]
pub fn assert_store_parse_as_eq<T, As, C>(value: T)
where
    As: CellSerializeAs<T> + CellDeserializeAsOwned<T, C>,
    T: PartialEq + Debug,
    for<'a> &'a Cell: TryInto<&'a C>,
    C: CellBehavior
{
    assert_eq!(
        value
            .wrap_as::<As>()
            .to_cell()
            .unwrap()
            .parse_fully_as::<T, As, C>()
            .unwrap(),
        value
    )
}
