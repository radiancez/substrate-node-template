use crate::{utils, Config};
use codec::Encode;
use frame_support::inherent::Vec;
use sp_runtime::offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef};

pub(crate) fn test_storage_access<T: Config>(block_number: T::BlockNumber) {
	// 奇数块写入，偶数块取出
	if utils::is_odd_block_number::<T>(block_number) {
		// 写 sp_runtime::offchain::storage::StorageValueRef
		set_storage_value::<T>(block_number);
		mutate_storage_value::<T>(block_number);
	} else {
		// 读 sp_runtime::offchain::storage::StorageValueRef
		get_storage_value::<T>(block_number);
	}
}

fn set_storage_value<T: Config>(block_number: T::BlockNumber) {
	let key = derived_key::<T>(block_number);
	let val_ref = StorageValueRef::persistent(&key);

	//  get a local random value
	let random_slice = sp_io::offchain::random_seed();

	//  get a local timestamp
	let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();

	// combine to a tuple and print it
	let value = (random_slice, timestamp_u64);

	//  write tuple content to key
	val_ref.set(&value);
	log::info!("[ {:?} ] set StorageValueRef: {:?}", block_number, value);
}

pub(crate) fn mutate_storage_value<T: Config>(block_number: T::BlockNumber) {
	let key = derived_key::<T>(block_number);
	let val_ref = StorageValueRef::persistent(&key);

	//  get a local random value
	let random_slice = sp_io::offchain::random_seed();

	//  get a local timestamp
	let timestamp_u64 = sp_io::offchain::timestamp().unix_millis();

	// combine to a tuple and print it
	let value = (random_slice, timestamp_u64);

	//  mutate tuple content to key
	struct StateError;
	let res = val_ref.mutate(
		|val: Result<Option<([u8; 32], u64)>, StorageRetrievalError>| -> Result<_, StateError> {
			match val {
				Ok(Some(_)) => Ok(value),
				_ => Ok(value),
			}
		},
	);

	match res {
		Ok(value) => {
			log::info!("[ {:?} ] mutate StorageValueRef successfully: {:?}", block_number, value);
		},
		Err(MutateStorageError::ValueFunctionFailed(_)) => {
			log::warn!("[ {:?} ] mutate StorageValueRef failed: {:?}", block_number, value);
		},
		Err(MutateStorageError::ConcurrentModification(_)) => {
			log::warn!("[ {:?} ] mutate StorageValueRef failed: {:?}", block_number, value);
		},
	}
}

fn get_storage_value<T: Config>(block_number: T::BlockNumber) {
	let key = derived_key::<T>(block_number - 1u32.into());
	let mut val_ref = StorageValueRef::persistent(&key);

	// get from db by key
	if let Ok(Some(value)) = val_ref.get::<([u8; 32], u64)>() {
		// print values
		log::info!("[ {:?} ] get StorageValueRef: {:?}", block_number, value);
		// delete that key
		val_ref.clear();
	}
}

#[deny(clippy::clone_double_ref)]
fn derived_key<T: Config>(block_number: T::BlockNumber) -> Vec<u8> {
	block_number.using_encoded(|encoded_bn| {
		b"node-template::storage::"
			.iter()
			.chain(encoded_bn)
			.copied()
			.collect::<Vec<u8>>()
	})
}
