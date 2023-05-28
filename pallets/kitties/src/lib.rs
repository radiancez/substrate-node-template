#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet_mod::*;

#[frame_support::pallet]
mod pallet_mod {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type KittyId = u32;
	#[derive(
		Clone, Copy, PartialEq, Eq, Default, TypeInfo, Encode, Decode, MaxEncodedLen, RuntimeDebug,
	)]
	pub struct Kitty(pub [u8; 16]);

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	// config

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
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
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId)>;

	////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	// event & error

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated { operator: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBred { operator: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransferred { operator: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
	}

	#[pallet::error]
	pub enum Error<T> {
		KittyIdOverflow,
		SameParentKittyId,
		KittyNotExist,
		NotKittyOwner,
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
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			let kitty_id = Self::generate_next_kitty_id()?;
			let kitty = Kitty(Self::random_genes(&operator));

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &operator);
			Self::deposit_event(Event::KittyCreated { operator, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn bred_kitty(
			origin: OriginFor<T>,
			parent_id_1: KittyId,
			parent_id_2: KittyId,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			ensure!(parent_id_1 != parent_id_2, Error::<T>::SameParentKittyId);
			let parent_1 = Self::kitties(parent_id_1).ok_or(Error::<T>::KittyNotExist)?;
			let parent_2 = Self::kitties(parent_id_2).ok_or(Error::<T>::KittyNotExist)?;

			let selector = Self::random_genes(&operator);
			let mut genes = <[u8; 16]>::default();
			for i in 0..parent_1.0.len() {
				genes[i] = (parent_1.0[i] & selector[i]) | (parent_2.0[i] & !selector[i])
			}

			let kitty_id = Self::generate_next_kitty_id()?;
			let kitty = Kitty(genes);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &operator);
			KittyParents::<T>::insert(kitty_id, (parent_id_1, parent_id_2));
			Self::deposit_event(Event::KittyBred { operator, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let operator = ensure_signed(origin)?;

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::KittyNotExist)?;
			ensure!(operator == owner, Error::<T>::NotKittyOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransferred { operator, recipient, kitty_id });
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

		fn random_genes(_: &T::AccountId) -> [u8; 16] {
			<[u8; 16]>::default()
		}
	}
}
