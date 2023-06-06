use crate::KittyDna;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

#[derive(Clone, PartialEq, Eq, Default, TypeInfo, Encode, Decode, MaxEncodedLen, RuntimeDebug)]
pub struct Kitty(pub KittyDna);
