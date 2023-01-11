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

	/// Service info
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
	pub struct ServiceInfo<ServiceId, IPAddress, SwarmKey, StatusFile, RFFFile, AccountId> {
		/// scribe
		pub scribe: AccountId,
		/// key
		pub service: AccountId,
		/// id
		pub id: ServiceId,
		/// IP address
		pub ip_address: IPAddress,
		/// swarm-key
		pub swarm_key: SwarmKey,
		/// status-file
		pub status: StatusFile,
		/// rff-file
		pub rff: RFFFile,
		/// deleted
		pub deleted: bool,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The maximum size of a SRC's KURI
		type MaxKURIlength: Get<u32>;
		/// The maximum size of a Service's ID
		type MaxServiceIdLength: Get<u32>;
		/// The maximum size of a Service's IP address
		type MaxIPAddressLength: Get<u32>;
		/// The maximum size of a Service's swarm-key
		type MaxServiceSwarmKeyLength: Get<u32>;
		/// The maximum size of a Service's status-file
		type MaxServiceStatusFileLength: Get<u32>;
		/// The maximum size of a Service's rff-file
		type MaxServiceRFFFileLength: Get<u32>;
	}

	pub type KURI<T> = BoundedVec<u8, <T as Config>::MaxKURIlength>;
	pub type ServiceId<T> = BoundedVec<u8, <T as Config>::MaxServiceIdLength>;
	pub type IPAddress<T> = BoundedVec<u8, <T as Config>::MaxIPAddressLength>;
	pub type SwarmKey<T> = BoundedVec<u8, <T as Config>::MaxServiceSwarmKeyLength>;
	pub type StatusFile<T> = BoundedVec<u8, <T as Config>::MaxServiceStatusFileLength>;
	pub type RFFFile<T> = BoundedVec<u8, <T as Config>::MaxServiceRFFFileLength>;

	pub type SRCInfoOf<T> = SRCInfo<
		KURI<T>,
		<T as frame_system::Config>::AccountId,
	>;
	pub type ServiceInfoOf<T> = ServiceInfo<
		ServiceId<T>,
		IPAddress<T>,
		SwarmKey<T>,
		StatusFile<T>,
		RFFFile<T>,
		<T as frame_system::Config>::AccountId,
	>;

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn total_srcs_created)]
	pub type TotalSRCsCreated<T> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn total_services_created)]
	pub type TotalServicesCreated<T> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn is_authorized_scribe)]
	pub(super) type AuthorizedScribeMap<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_service)]
	pub(super) type AvailableServiceMap<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_src_transfer_accepted)]
	pub(super) type SRCTransferAccepted<T: Config> = StorageDoubleMap<_, Blake2_128Concat, KURI<T>, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn srcs)]
	pub(super) type SRCs<T: Config> = StorageMap<_, Blake2_128Concat, KURI<T>, SRCInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn services)]
	pub(super) type Services<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ServiceInfoOf<T>>;

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
		SRCDeleted(BoundedVec<u8, T::MaxKURIlength>),
		SRCUpdated(BoundedVec<u8, T::MaxKURIlength>),
		SRCTransferAccepted(BoundedVec<u8, T::MaxKURIlength>, T::AccountId),
		SRCTransferred(BoundedVec<u8, T::MaxKURIlength>),
		ServiceCreated(T::AccountId),
		ServiceDeleted(T::AccountId),
		ServiceUpdated(T::AccountId),
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
		/// SRC has already been added.
		SRCAlreadyAdded,
		/// SRC has already been deleted.
		SRCAlreadyDeleted,
		/// A service ID is longer than the allowed limit.
		MaxServiceIdLengthExceeded,
		/// A service IP address is longer than the allowed limit.
		MaxIPAddressLengthExceeded,
		/// A service swarm key is longer than the allowed limit.
		MaxServiceSwarmKeyLengthExceeded,
		/// A service status file is longer than the allowed limit.
		MaxServiceStatusFileLengthExceeded,
		/// A service RFF file is longer than the allowed limit.
		MaxServiceRFFFileLengthExceeded,
		/// Service not found.
		ServiceNotFound,
		/// Service has already been added.
		ServiceAlreadyAdded,
		/// Service has already been deleted.
		ServiceAlreadyDeleted,
	}

	// Dispatchable functions to interact with the pallet and invoke state changes.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		
		////// SRC FUNCTIONS //////
		
		
		/// As a scribe ///

		/// Register a new SRC.
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn self_register_content(origin: OriginFor<T>, kuri: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_authorized_scribe(&signer), Error::<T>::CallForbidden);

			let bounded_kuri: BoundedVec<u8, T::MaxKURIlength> =
			kuri.try_into().map_err(|_| Error::<T>::MaxKURILengthExceeded)?;

			ensure!(SRCs::<T>::get(bounded_kuri.clone()) == None, Error::<T>::SRCAlreadyAdded);

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

		/// Accept a transfer of a SRC.
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

		/// Transfer a SRC to another account.
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

		/// Delete a SRC.
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


		////// SERVICE FUNCTIONS //////

		
		/// As a scribe ///

		/// Register a service.
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn register_service(origin: OriginFor<T>, service_address: T::AccountId, service_id: Vec<u8>, ip_address: Vec<u8>, swarm_key: Vec<u8>, status_file: Vec<u8>, rff_file: Vec<u8>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_authorized_scribe(&signer), Error::<T>::CallForbidden);

			let bounded_service_id: BoundedVec<u8, T::MaxServiceIdLength> =
			service_id.try_into().map_err(|_| Error::<T>::MaxServiceIdLengthExceeded)?;
			let bounded_ip_address: BoundedVec<u8, T::MaxIPAddressLength> =
			ip_address.try_into().map_err(|_| Error::<T>::MaxIPAddressLengthExceeded)?;
			let bounded_swarm_key: BoundedVec<u8, T::MaxServiceSwarmKeyLength> =
			swarm_key.try_into().map_err(|_| Error::<T>::MaxServiceSwarmKeyLengthExceeded)?;
			let bounded_status_file: BoundedVec<u8, T::MaxServiceStatusFileLength> =
			status_file.try_into().map_err(|_| Error::<T>::MaxServiceStatusFileLengthExceeded)?;
			let bounded_rff_file: BoundedVec<u8, T::MaxServiceRFFFileLength> =
			rff_file.try_into().map_err(|_| Error::<T>::MaxServiceRFFFileLengthExceeded)?;

			ensure!(Services::<T>::get(service_address.clone()) == None, Error::<T>::ServiceAlreadyAdded);

			let info = ServiceInfo {
				scribe: signer.clone(),
				service: service_address.clone(),
				id: bounded_service_id.clone(),
				ip_address: bounded_ip_address.clone(),
				swarm_key: bounded_swarm_key.clone(),
				status: bounded_status_file.clone(),
				rff: bounded_rff_file.clone(),
				deleted: false,
			};

			// Update storage.

			Services::<T>::insert(service_address.clone(), info);

			match <TotalServicesCreated<T>>::get() {
				None => {
					<TotalServicesCreated<T>>::put(1);
				}
				Some(old) => {
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					<TotalServicesCreated<T>>::put(new);
				},
			}

			// Emit an event.
			Self::deposit_event(Event::ServiceCreated(service_address));
			Ok(())
		}

		/// Update a service.
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn update_service(origin: OriginFor<T>, service_address: T::AccountId, service_id: Vec<u8>, ip_address: Vec<u8>, swarm_key: Vec<u8>, status_file: Vec<u8>, rff_file: Vec<u8>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_authorized_scribe(&signer), Error::<T>::CallForbidden);
			
			Services::<T>::try_mutate_exists(service_address.clone(), |service_info| -> DispatchResult {
				let info = service_info.as_mut().ok_or(Error::<T>::ServiceNotFound)?;
				ensure!(info.scribe == signer, Error::<T>::CallForbidden);
				ensure!(info.deleted.eq(&false), Error::<T>::ServiceAlreadyDeleted);
				let bounded_service_id: BoundedVec<u8, T::MaxServiceIdLength> =
				service_id.try_into().map_err(|_| Error::<T>::MaxServiceIdLengthExceeded)?;
				let bounded_ip_address: BoundedVec<u8, T::MaxIPAddressLength> =
				ip_address.try_into().map_err(|_| Error::<T>::MaxIPAddressLengthExceeded)?;
				let bounded_swarm_key: BoundedVec<u8, T::MaxServiceSwarmKeyLength> =
				swarm_key.try_into().map_err(|_| Error::<T>::MaxServiceSwarmKeyLengthExceeded)?;
				let bounded_status_file: BoundedVec<u8, T::MaxServiceStatusFileLength> =
				status_file.try_into().map_err(|_| Error::<T>::MaxServiceStatusFileLengthExceeded)?;
				let bounded_rff_file: BoundedVec<u8, T::MaxServiceRFFFileLength> =
				rff_file.try_into().map_err(|_| Error::<T>::MaxServiceRFFFileLengthExceeded)?;
				info.id = bounded_service_id.clone();
				info.ip_address = bounded_ip_address.clone();
				info.swarm_key = bounded_swarm_key.clone();
				info.status = bounded_status_file.clone();
				info.rff = bounded_rff_file.clone();
				Self::deposit_event(Event::ServiceUpdated(service_address));
				Ok(())
			})
		}

		/// Delete a service.
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn delete_service(origin: OriginFor<T>, service_address: T::AccountId) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(Self::is_authorized_scribe(&signer), Error::<T>::CallForbidden);
			
			Services::<T>::try_mutate_exists(service_address.clone(), |service_info| -> DispatchResult {
				let info = service_info.as_mut().ok_or(Error::<T>::ServiceNotFound)?;
				ensure!(info.scribe == signer, Error::<T>::CallForbidden);
				ensure!(info.deleted.eq(&false), Error::<T>::ServiceAlreadyDeleted);
				info.deleted = true;
	
				Self::deposit_event(Event::ServiceDeleted(service_address));
				Ok(())
			})
		}


		/// As a service ///

		/// Update own status
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn update_service_status(origin: OriginFor<T>, status_file: Vec<u8>, rff_file: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let signer = ensure_signed(origin)?;

			ensure!(Self::is_service(&signer), Error::<T>::CallForbidden);

			// Update storage.
			Services::<T>::try_mutate_exists(signer.clone(), |service_info| -> DispatchResult {
				let info = service_info.as_mut().ok_or(Error::<T>::ServiceNotFound)?;
				ensure!(info.deleted.eq(&false), Error::<T>::ServiceAlreadyDeleted);

				let bounded_status_file: BoundedVec<u8, T::MaxServiceStatusFileLength> =
				status_file.try_into().map_err(|_| Error::<T>::MaxServiceStatusFileLengthExceeded)?;
				let bounded_rff_file: BoundedVec<u8, T::MaxServiceRFFFileLength> =
				rff_file.try_into().map_err(|_| Error::<T>::MaxServiceRFFFileLengthExceeded)?;
				info.status = bounded_status_file.clone();
				info.rff = bounded_rff_file.clone();
	
				Self::deposit_event(Event::ServiceDeleted(signer));
				Ok(())
			})
		}


		////// SCRIBE FUNCTIONS //////
		
		
		/// As a scribe ///

		/// Update another scribe's status
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


		/// ROOT FUNCTIONS ///


		/// Update a scribe's status
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

		/// Update a SRC's info
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn force_update_src_info(origin: OriginFor<T>, kuri: Vec<u8>, owner: T::AccountId, scribe: T::AccountId, deleted: bool) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_root(origin)?;
			
			let bounded_kuri: BoundedVec<u8, T::MaxKURIlength> =
			kuri.try_into().map_err(|_| Error::<T>::MaxKURILengthExceeded)?;

			let info = SRCInfo {
				kuri: bounded_kuri.clone(),
				owner,
				scribe,
				deleted,
			};

			// Update storage.
			SRCs::<T>::insert(bounded_kuri.clone(), info);

			Self::deposit_event(Event::SRCUpdated(bounded_kuri));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Update a service's info
		#[pallet::weight((10_000 + T::DbWeight::get().writes(1).ref_time(), Pays::No))]
		pub fn force_update_service_info(origin: OriginFor<T>, scribe: T::AccountId, service_address: T::AccountId, service_id: Vec<u8>, ip_address: Vec<u8>, swarm_key: Vec<u8>, status_file: Vec<u8>, rff_file: Vec<u8>, deleted: bool) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			ensure_root(origin)?;
			
			let bounded_service_id: BoundedVec<u8, T::MaxServiceIdLength> =
			service_id.try_into().map_err(|_| Error::<T>::MaxServiceIdLengthExceeded)?;
			let bounded_ip_address: BoundedVec<u8, T::MaxIPAddressLength> =
			ip_address.try_into().map_err(|_| Error::<T>::MaxIPAddressLengthExceeded)?;
			let bounded_swarm_key: BoundedVec<u8, T::MaxServiceSwarmKeyLength> =
			swarm_key.try_into().map_err(|_| Error::<T>::MaxServiceSwarmKeyLengthExceeded)?;
			let bounded_status_file: BoundedVec<u8, T::MaxServiceStatusFileLength> =
			status_file.try_into().map_err(|_| Error::<T>::MaxServiceStatusFileLengthExceeded)?;
			let bounded_rff_file: BoundedVec<u8, T::MaxServiceRFFFileLength> =
			rff_file.try_into().map_err(|_| Error::<T>::MaxServiceRFFFileLengthExceeded)?;

			let info = ServiceInfo {
				scribe,
				service: service_address.clone(),
				id: bounded_service_id.clone(),
				ip_address: bounded_ip_address.clone(),
				swarm_key: bounded_swarm_key.clone(),
				status: bounded_status_file.clone(),
				rff: bounded_rff_file.clone(),
				deleted,
			};

			// Update storage.
			Services::<T>::insert(service_address.clone(), info);

			Self::deposit_event(Event::ServiceUpdated(service_address));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

	}
}
