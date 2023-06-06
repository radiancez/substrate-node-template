pub use v2 as current_version; // 当前版本
pub mod v2;

use crate::{Config, Kitties, Pallet};
use frame_support::{
	migration::storage_key_iter, pallet_prelude::*, traits::GetStorageVersion, weights::Weight,
	StoragePrefixedMap,
};
mod v0;
mod v1;

pub(crate) fn upgrade_storage<T: Config>() -> Weight {
	let current_version = Pallet::<T>::current_storage_version();
	if current_version != current_version::STORAGE_VERSION {
		return Weight::zero()
	}

	let on_chain_version: StorageVersion = Pallet::<T>::on_chain_storage_version();
	if on_chain_version == v1::STORAGE_VERSION {
		v1_to_v2::<T>();
	} else if on_chain_version == v0::STORAGE_VERSION {
		v0_to_v2::<T>();
	}

	Weight::zero()
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// v1 -> v2

fn v1_to_v2<T: Config>() {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (kitty_id, kitty_old) in
		storage_key_iter::<v1::KittyId, v1::Kitty, Blake2_128Concat>(module, item).drain()
	{
		let kitty = v2::Kitty { name: name_v1_to_v2(&kitty_old.name, b"5678"), dna: kitty_old.dna };
		Kitties::<T>::insert(kitty_id, &kitty);
	}
}

fn name_v1_to_v2(name_v1: &v1::KittyName, append: &[u8; 4]) -> v2::KittyName {
	let mut result = [0; 8];
	result[..4].copy_from_slice(name_v1);
	result[4..].copy_from_slice(append);
	result
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// v0 -> v2

fn v0_to_v2<T: Config>() {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (kitty_id, kitty_old) in
		storage_key_iter::<v0::KittyId, v0::Kitty, Blake2_128Concat>(module, item).drain()
	{
		let kitty = v2::Kitty { name: *b"12345678", dna: kitty_old.0 };
		Kitties::<T>::insert(kitty_id, &kitty);
	}
}
