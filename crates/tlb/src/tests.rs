use core::fmt::Debug;

use crate::{
    de::r#as::CellDeserializeAsOwned,
    ser::{
        r#as::{CellSerializeAs, CellSerializeWrapAsExt},
        CellSerializeExt,
    },
};
use crate::cell::CellBehavior;

#[track_caller]
pub fn assert_store_parse_as_eq<T, As>(value: T)
where
    As: CellSerializeAs<T> + CellDeserializeAsOwned<T>,
    T: PartialEq + Debug,
{
    assert_eq!(
        value
            .wrap_as::<As>()
            .to_cell()
            .unwrap()
            .as_ordinary()
            .unwrap()
            .parse_fully_as::<T, As>()
            .unwrap(),
        value
    )
}
