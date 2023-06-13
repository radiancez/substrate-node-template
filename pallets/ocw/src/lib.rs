#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod off_chain;
mod utils;

#[frame_support::pallet]
mod pallet {
	use crate::{off_chain, utils};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	////////////////////////////////////////////////////////////////////////////////////////////////////
	/// config

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////
	/// storage

	////////////////////////////////////////////////////////////////////////////////////////////////////
	/// event & error

	#[pallet::event]
	// #[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	////////////////////////////////////////////////////////////////////////////////////////////////////
	/// pallet

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			log::info!("[ {:?} ] on_initialize", block_number);
			Weight::from_parts(0, 0)
		}

		fn on_finalize(block_number: T::BlockNumber) {
			log::info!("[ {:?} ] on_finalize", block_number);
		}

		fn on_idle(block_number: T::BlockNumber, remaining_weight: Weight) -> Weight {
			log::info!("[ {:?} ] on_idle, {:?}", block_number, remaining_weight);
			Weight::from_parts(0, 0)
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("[ {:?} ] offchain_worker enter", block_number);

			// 奇数块写入，偶数块取出
			if utils::is_odd_block_number::<T>(block_number) {
				// 写 sp_runtime::offchain::storage::StorageValueRef
				off_chain::set_storage_value::<T>(block_number);
				off_chain::mutate_storage_value::<T>(block_number);
			} else {
				// 读 sp_runtime::offchain::storage::StorageValueRef
				off_chain::get_storage_value::<T>(block_number);
			}

			// 隔断一下，日志看得更清晰
			log::info!("[ {:?} ] ====================================================================================================", block_number);

			utils::sleep(8000); // 推迟 offchain_worker leave，证明 offchain_worker 生命周期与出块是解耦的
			log::info!("[ {:?} ] offchain_worker leave", block_number);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
