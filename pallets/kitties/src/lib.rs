#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet, pallet_prelude::*, traits::Randomness, Hashable};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub type KittyId = u32;

	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parent)]
	pub type KittyParent<T> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateKittyEvent { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyBreed { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
		KittyTransfered { who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
	}
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_100 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty(Self::random_value(&who));
			Kitties::<T>::insert(&kitty_id, &kitty);
			KittyOwner::<T>::insert(&kitty_id, &who);
			Self::deposit_event(Event::CreateKittyEvent { who, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_100 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id1: KittyId,
			kitty_id2: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id1 != kitty_id2, Error::<T>::SameKittyId);
			ensure!(Kitties::<T>::contains_key(&kitty_id1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(&kitty_id2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?;
			let kitty1 = Self::kitties(kitty_id1)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
			let kitty2 = Self::kitties(kitty_id2)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			let selector = Self::random_value(&who);
			let mut data = [0u8; 16];
			for i in 0..kitty1.0.len() {
				data[i] = kitty1.0[i] & selector[i] | (kitty2.0[i] & !selector[i]);
			}
			let kitty = Kitty(data);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParent::<T>::insert(kitty_id, (kitty_id1, kitty_id2));
			Self::deposit_event(Event::KittyBreed { who, kitty_id, kitty });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_100 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(&kitty_id), Error::<T>::InvalidKittyId);

			let owner = Self::kitty_owner(kitty_id)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
			ensure!(owner == who, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransfered { who, recipient, kitty_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}
	}
}
