#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet_mod::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
mod pallet_mod {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	// config

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	// storage

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	// event & error

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated{signer: T::AccountId, claim: BoundedVec<u8, T::MaxClaimLength>},
		ClaimRevoked{signer: T::AccountId, claim: BoundedVec<u8, T::MaxClaimLength>},
		ClaimTransferred{signer: T::AccountId, recipient: T::AccountId, claim: BoundedVec<u8, T::MaxClaimLength>},
	}

	#[pallet::error]
	pub enum Error<T> {
		ClaimAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		TransferToOwner,
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	// pallet

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ClaimAlreadyExist);

			Proofs::<T>::insert(
				&claim,
				(signer.clone(), frame_system::Pallet::<T>::block_number()),
			);
			Self::deposit_event(Event::ClaimCreated{signer, claim});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let (owner, _block_number) =
				Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == signer, Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&claim);
			Self::deposit_event(Event::ClaimRevoked{signer, claim});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let (owner, _block_number) =
				Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(signer == owner, Error::<T>::NotClaimOwner);
			ensure!(signer != recipient, Error::<T>::TransferToOwner);

			Proofs::<T>::insert(&claim, (recipient.clone(), frame_system::Pallet::<T>::block_number()));
			Self::deposit_event(Event::ClaimTransferred{signer, recipient, claim});
			Ok(())
		}
	}
}
