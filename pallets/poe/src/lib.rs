#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

	#[pallet::storage]
    pub type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxClaimLength>, (T::AccountId, T::BlockNumber)>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRemoved(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofAlreadyExist,
        ClaimTooLong,
        ClaimNotExist,
        NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        #[pallet::call_index(0)]
        pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
            ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

            Proofs::<T>::insert(
                &bounded_claim,
                (sender.clone(), frame_system::Pallet::<T>::block_number()),
            );

            // Emit an event.
            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        } 

		#[pallet::weight(0)]
        #[pallet::call_index(1)]
        pub fn remove_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
            let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

            Proofs::<T>::remove(&bounded_claim);

            // Emit an event.
            Self::deposit_event(Event::ClaimRemoved(sender, claim));

            Ok(().into())
        }

		#[pallet::weight(0)]
        #[pallet::call_index(2)]
        pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
            let (owner, _block_number) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::insert(
                &bounded_claim,
                (dest, frame_system::Pallet::<T>::block_number()),
            );

            // Emit an event.
            Self::deposit_event(Event::ClaimRemoved(sender, claim));

            Ok(().into())
        }
    } 
}
