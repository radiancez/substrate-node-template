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
	use frame_system::{
		offchain::{
			AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
			SigningTypes,
		},
		pallet_prelude::*,
	};

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct Payload<Public> {
		number: u64,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	////////////////////////////////////////////////////////////////////////////////////////////////////
	/// config

	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		// The identifier type for an offchain worker.
		type AppCrypto: AppCrypto<Self::Public, Self::Signature>;
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

			let number: u64 = 42;
			// Retrieve the signer to sign the payload
			let signer = Signer::<T, T::AppCrypto>::any_account();

			// `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(),
			// ()>)>`. 	 The returned result means:
			// 	 - `None`: no account is available for sending transaction
			// 	 - `Some((account, Ok(())))`: transaction is successfully sent
			// 	 - `Some((account, Err(())))`: error occurred when sending the transaction
			if let Some((_, res)) = signer.send_unsigned_transaction(
				// this line is to prepare and return payload
				|acct| Payload { number, public: acct.public.clone() },
				|payload, signature| Call::unsigned_extrinsic_with_signed_payload {
					payload,
					signature,
				},
			) {
				match res {
					Ok(()) => {
						log::info!(
							"[ {:?} ] unsigned tx with signed payload successfully sent.",
							block_number
						);
					},
					Err(()) => {
						log::error!(
							"[ {:?} ] sending unsigned tx with signed payload failed.",
							block_number
						);
					},
				};
			} else {
				// The case of `None`: no account is available for sending
				log::error!("[ {:?} ] No local account available", block_number);
			}

			// 隔断一下，日志看得更清晰
			log::info!("[ {:?} ] ====================================================================================================", block_number);
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const UNSIGNED_TXS_PRIORITY: u64 = 100;
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("my-pallet")
					.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::unsigned_extrinsic_with_signed_payload { ref payload, ref signature } => {
					if !SignedPayload::<T>::verify::<T::AppCrypto>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into()
					}
					valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn offchain_index_set(origin: OriginFor<T>, number: u64) -> DispatchResult {
			let _signer = ensure_signed(origin)?;

			// offchain index 写入 local storage
			crate::offchain::indexing::offchain_index_set(
				frame_system::Pallet::<T>::block_number(),
				number,
			);

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn unsigned_extrinsic_with_signed_payload(
			origin: OriginFor<T>,
			payload: Payload<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::info!(
				"[ {:?} ] in call unsigned_extrinsic_with_signed_payload: {:?}",
				frame_system::Pallet::<T>::block_number(),
				payload.number,
			);

			Ok(())
		}
	}
}
