#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// SRC info
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
	pub struct SRCInfo<KURI, AccountId> {
		/// SRC kuri
		pub kuri: KURI,
		/// SRC owner
		pub owner: AccountId,
		/// SRC scribe
		pub scribe: AccountId,
		/// SRC status
		pub deleted: bool,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The maximum size of a SRC's KURI
		type MaxKURIlength: Get<u32>;
	}

	pub type KURI<T> = BoundedVec<u8, <T as Config>::MaxKURIlength>;

	pub type SRCInfoOf<T> = SRCInfo<
		KURI<T>,
		<T as frame_system::Config>::AccountId,
	>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn total_srcs_created)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type TotalSRCsCreated<T> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn is_authorized_scribe)]
	pub(super) type AuthorizedScribeMap<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_src_transfer_accepted)]
	pub(super) type SRCTransferAccepted<T: Config> = StorageDoubleMap<_, Blake2_128Concat, KURI<T>, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn srcs)]
	pub(super) type SRCs<T: Config> = StorageMap<_, Blake2_128Concat, KURI<T>, SRCInfoOf<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ScribeAdded(T::AccountId, T::AccountId),
		ScribeRemoved(T::AccountId, T::AccountId),
		ScribeAddedByAdmin(T::AccountId),
		ScribeRemovedByAdmin(T::AccountId),
		SRCCreated(BoundedVec<u8, T::MaxKURIlength>),
		SRCTransferAccepted(BoundedVec<u8, T::MaxKURIlength>, T::AccountId),
		SRCTransferred(BoundedVec<u8, T::MaxKURIlength>),
		SRCDeleted(BoundedVec<u8, T::MaxKURIlength>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		CallForbidden,
		/// A KURI is longer than the allowed limit.
		MaxKURILengthExceeded,
		/// 404 for a SRC.
		SRCNotFound,
		/// SRC has already been deleted.
		SRCAlreadyDeleted,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn self_register_content(origin: OriginFor<T>, kuri: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_authorized_scribe(&signer), Error::<T>::CallForbidden);

			let bounded_kuri: BoundedVec<u8, T::MaxKURIlength> =
			kuri.try_into().map_err(|_| Error::<T>::MaxKURILengthExceeded)?;

			let info = SRCInfo {
				kuri: bounded_kuri.clone(),
				owner: signer.clone(),
				scribe: signer.clone(),
				deleted: false,
			};

			// Update storage.

			SRCs::<T>::insert(bounded_kuri.clone(), info);

			match <TotalSRCsCreated<T>>::get() {
				None => {
					<TotalSRCsCreated<T>>::put(1);
				}
				Some(old) => {
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					<TotalSRCsCreated<T>>::put(new);
				},
			}

			// Emit an event.
			Self::deposit_event(Event::SRCCreated(bounded_kuri));
			Ok(())
		}

		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn accept_src_transfer(origin: OriginFor<T>, kuri: Vec<u8>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let bounded_kuri: BoundedVec<u8, T::MaxKURIlength> =
			kuri.try_into().map_err(|_| Error::<T>::MaxKURILengthExceeded)?;
			SRCs::<T>::try_mutate_exists(bounded_kuri.clone(), |src_info| -> DispatchResult {
				let info = src_info.as_mut().ok_or(Error::<T>::SRCNotFound)?;
				ensure!(info.owner != signer, Error::<T>::CallForbidden);
				ensure!(info.deleted.eq(&false), Error::<T>::SRCAlreadyDeleted);
				// Update storage.
				<SRCTransferAccepted<T>>::insert(bounded_kuri.clone(), &signer, true);
	
				Self::deposit_event(Event::SRCTransferAccepted(bounded_kuri, signer));
				Ok(())
			})
		}

		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn transfer_src(origin: OriginFor<T>, to: T::AccountId, kuri: Vec<u8>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let bounded_kuri: BoundedVec<u8, T::MaxKURIlength> =
			kuri.try_into().map_err(|_| Error::<T>::MaxKURILengthExceeded)?;
			ensure!(Self::is_src_transfer_accepted(bounded_kuri.clone(), &to), Error::<T>::CallForbidden);
			SRCs::<T>::try_mutate_exists(bounded_kuri.clone(), |src_info| -> DispatchResult {
				let info = src_info.as_mut().ok_or(Error::<T>::SRCNotFound)?;
				ensure!(info.owner == signer, Error::<T>::CallForbidden);
				ensure!(info.deleted.eq(&false), Error::<T>::SRCAlreadyDeleted);
				info.owner = to;
	
				Self::deposit_event(Event::SRCTransferred(bounded_kuri));
				Ok(())
			})
		}

		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn delete_src(origin: OriginFor<T>, kuri: Vec<u8>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let bounded_kuri: BoundedVec<u8, T::MaxKURIlength> =
			kuri.try_into().map_err(|_| Error::<T>::MaxKURILengthExceeded)?;
			SRCs::<T>::try_mutate_exists(bounded_kuri.clone(), |src_info| -> DispatchResult {
				let info = src_info.as_mut().ok_or(Error::<T>::SRCNotFound)?;
				ensure!(info.owner == signer, Error::<T>::CallForbidden);
				ensure!(info.deleted.eq(&false), Error::<T>::SRCAlreadyDeleted);
				info.deleted = true;
	
				Self::deposit_event(Event::SRCDeleted(bounded_kuri));
				Ok(())
			})
		}

		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn force_update_scribe_authority_status(origin: OriginFor<T>, scribe: T::AccountId, status: bool) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_root(origin)?;
			
			// Update storage.
			<AuthorizedScribeMap<T>>::insert(&scribe, status);

			// Emit an event.
			if status.eq(&true) {
				Self::deposit_event(Event::ScribeAddedByAdmin(scribe));
			}
			else {
				Self::deposit_event(Event::ScribeRemovedByAdmin(scribe));
			}
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn update_scribe_authority_status(origin: OriginFor<T>, scribe: T::AccountId, status: bool) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			ensure!(Self::is_authorized_scribe(&signer), Error::<T>::CallForbidden);

			// Update storage.
			<AuthorizedScribeMap<T>>::insert(&scribe, status);

			// Emit an event.
			if status.eq(&true) {
				Self::deposit_event(Event::ScribeAdded(signer, scribe));
			}
			else {
				Self::deposit_event(Event::ScribeRemoved(signer, scribe));
			}
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

	}
}
