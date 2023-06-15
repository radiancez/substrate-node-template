#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod offchain;

#[frame_support::pallet]
mod pallet {
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
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("[ {:?} ] offchain_worker", block_number);

			// 隔断一下，日志看得更清晰
			log::info!("[ {:?} ] ====================================================================================================", block_number);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn offchain_index_set(origin: OriginFor<T>, number: u64) -> DispatchResult {
			let _signer = ensure_signed(origin)?;

			// offchain index 写入 local storage
			crate::offchain::offchain_index_set(frame_system::Pallet::<T>::block_number(), number);

			Ok(())
		}
	}
}
