#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
mod pallet {
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	pub type KittyId = u32;
	pub type KittyGenes = [u8; 16];
	#[derive(
		Clone, Copy, PartialEq, Eq, Default, TypeInfo, Encode, Decode, MaxEncodedLen, RuntimeDebug,
	)]
	pub struct Kitty(pub KittyGenes);

	////////////////////////////////////////////////////////////////////////////////////////////////////
	// config

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type KittyGenesRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////
	// storage

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId)>;

	////////////////////////////////////////////////////////////////////////////////////////////////////
	// event & error

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { account: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBred { account: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { sender: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
	}

	#[pallet::error]
	pub enum Error<T> {
		KittyIdOverflow,
		SameParentKittyId,
		KittyNotExist,
		NotKittyOwner,
		TransferKittyToOwner,
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////
	// pallet

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let kitty_id = Self::generate_next_kitty_id()?;
			let kitty = Kitty(Self::random_kitty_genes(&signer));

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &signer);
			Self::deposit_event(Event::KittyCreated { account: signer, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn bred_kitty(
			origin: OriginFor<T>,
			parent_id_1: KittyId,
			parent_id_2: KittyId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			ensure!(parent_id_1 != parent_id_2, Error::<T>::SameParentKittyId);
			let parent_1 = Self::kitties(parent_id_1).ok_or(Error::<T>::KittyNotExist)?;
			let parent_2 = Self::kitties(parent_id_2).ok_or(Error::<T>::KittyNotExist)?;
			let kitty_id = Self::generate_next_kitty_id()?;
			let kitty = Kitty(Self::child_kitty_genes(&signer, &parent_1, &parent_2));

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &signer);
			KittyParents::<T>::insert(kitty_id, (parent_id_1, parent_id_2));
			Self::deposit_event(Event::KittyBred { account: signer, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer_kitty(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::KittyNotExist)?;
			ensure!(signer == owner, Error::<T>::NotKittyOwner);
			ensure!(signer != recipient, Error::<T>::TransferKittyToOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransferred { sender: signer, recipient, kitty_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn generate_next_kitty_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_kitty_id| -> Result<KittyId, DispatchError> {
				let kitty_id = *next_kitty_id;
				*next_kitty_id = kitty_id.checked_add(1).ok_or(Error::<T>::KittyIdOverflow)?;
				Ok(kitty_id)
			})
		}

		pub(crate) fn random_kitty_genes(account: &T::AccountId) -> KittyGenes {
			let payload = (
				T::KittyGenesRandomness::random_seed(),
				&account,
				frame_system::Pallet::<T>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		pub(crate) fn child_kitty_genes(
			account: &T::AccountId,
			parent_1: &Kitty,
			parent_2: &Kitty,
		) -> KittyGenes {
			let selector = Self::random_kitty_genes(&account);
			let mut genes = KittyGenes::default();
			for i in 0..parent_1.0.len() {
				genes[i] = (parent_1.0[i] & selector[i]) | (parent_2.0[i] & !selector[i])
			}
			return genes
		}
	}
}
