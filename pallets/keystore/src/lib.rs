#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

pub use pallet::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	use crate::weights::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
		/// The maximum size of a key.
		type MaxSize: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// This module's main storage will consist of a StorageDoubleMap connecting addresses to the
	/// list of keys they've submitted and not revoked.
	#[pallet::getter(fn key)]
	pub type IssuedKeys<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxSize>,
		BoundedVec<u8, T::MaxSize>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn encryption_key)]
	/// Maps an account to an encryption key that they've issued.
	pub type IssuedEncryptionKeys<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, [u8; 32]>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Announce when an identity has broadcast a new key as an event.
        KeyAnnounced { key: BoundedVec<u8, T::MaxSize>, who: T::AccountId },
		/// Announce when an identity has set a key as revoked.
        KeyRevoked { key: BoundedVec<u8, T::MaxSize>, who: T::AccountId },
        /// Announce that an encryption key was issued.
        EncryptionKeyIssued { who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The specified key already exists.
		KeyExists,
		/// The specified key does not exist.
		KeyDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Can we add documentation here?
		#[pallet::weight(T::WeightInfo::announce_key())]
		#[pallet::call_index(0)]
		pub fn announce_key(
			origin: OriginFor<T>,
			fingerprint: BoundedVec<u8, T::MaxSize>,
			location: BoundedVec<u8, T::MaxSize>,
        ) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(!<IssuedKeys<T>>::contains_key(&who, &fingerprint), Error::<T>::KeyExists);

			<IssuedKeys<T>>::insert(&who, &fingerprint, &location);
            Self::deposit_event(Event::KeyAnnounced { key: fingerprint.clone(), who: who.clone() });
            Ok(().into())
		}

		/// If a key needs to be removed from circulation, this extrinsic will handle deleting it
		/// and informing the network.
		#[pallet::weight(T::WeightInfo::revoke_key())]
		#[pallet::call_index(1)]
		pub fn revoke_key(
			origin: OriginFor<T>,
			key_index: BoundedVec<u8, T::MaxSize>,
        ) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(<IssuedKeys<T>>::contains_key(&who, &key_index), Error::<T>::KeyDoesNotExist);

			<IssuedKeys<T>>::remove(&who, &key_index);
            Self::deposit_event(Event::KeyRevoked { key: key_index.clone(), who: who.clone() });
            Ok(().into())
		}

		/// Announces an encryption key to the network.
		#[pallet::weight(T::WeightInfo::issue_encryption_key())]
		#[pallet::call_index(2)]
        pub fn issue_encryption_key(origin: OriginFor<T>, key: [u8; 32]) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<IssuedEncryptionKeys<T>>::insert(&who, key);
            Self::deposit_event(Event::EncryptionKeyIssued { who: who.clone() });
            Ok(().into())
		}
	}
}
