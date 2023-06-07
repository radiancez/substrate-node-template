use crate::{storage, Config, Kitties, KittyDna, KittyId, KittyName, Pallet};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	migration::storage_key_iter, pallet_prelude::*, traits::GetStorageVersion, weights::Weight,
	StoragePrefixedMap,
};
use scale_info::TypeInfo;

pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[derive(Clone, PartialEq, Eq, Default, TypeInfo, Encode, Decode, MaxEncodedLen, RuntimeDebug)]
pub struct Kitty {
	pub name: KittyName,
	pub dna: KittyDna,
}

pub fn upgrade<T: Config>() -> Weight {
	let current_version = Pallet::<T>::current_storage_version();
	if current_version != STORAGE_VERSION {
		return Weight::zero()
	}

	let on_chain_version: StorageVersion = Pallet::<T>::on_chain_storage_version();
	if on_chain_version == storage::v0::STORAGE_VERSION {
		v0_to_v1::<T>();
		STORAGE_VERSION.put::<Pallet::<T>>();
	}

	Weight::zero()
}

fn v0_to_v1<T: Config>() {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (kitty_id, kitty_v0) in
		storage_key_iter::<KittyId, storage::v0::Kitty, Blake2_128Concat>(module, item).drain()
	{
		let kitty = Kitty { name: *b"____", dna: kitty_v0.0 };
		Kitties::<T>::insert(kitty_id, &kitty);
	}	
}
