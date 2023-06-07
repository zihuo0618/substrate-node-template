#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod migrations;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Randomness, Currency, ExistenceRequirement};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use frame_support::{pallet, PalletId};
	use sp_runtime::traits::AccountIdConversion;
	use crate::migrations;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	pub type KittyId = u32;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct Kitty{
		pub dna: [u8; 16],
		pub name: [u8;8],
	}

	#[pallet::hooks]
	impl<T:Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			migrations::v2::migrate::<T>()
		}
	}


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

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
		KittyBought { who: T::AccountId, kitty_id: KittyId },
		KittyOnSale { who: T::AccountId, kitty_id: KittyId},
	}
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		AlreadyOwned,
		NotOnSale,
		AlreadyOnSale,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_100 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>, name: [u8;8]) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id()?;
			let dna = Self::random_value(&who);
			let kitty = Kitty {
				dna,
				name,
			};

			let price = T::KittyPrice::get();
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

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
			name: [u8;8],
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
			for i in 0..kitty1.dna.len() {
				data[i] = kitty1.dna[i] & selector[i] | (kitty2.dna[i] & !selector[i]);
			}
			let kitty = Kitty {
				dna: data,
				name,
			};

			let price = T::KittyPrice::get();
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

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

			let price = T::KittyPrice::get();
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransfered { who, recipient, kitty_id });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_100 + T::DbWeight::get().writes(1).ref_time())]
		pub fn buy(origin: OriginFor<T>, kitty_id: u32)-> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			let owner = Self::kitty_owner(kitty_id).ok_or::<DispatchError>(Error::<T>::NotOwner.into())?;
			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			T::Currency::transfer(&who, &owner	, price, ExistenceRequirement::KeepAlive)?;

			KittyOwner::<T>::insert(kitty_id, &who);
			KittyOnSale::<T>::remove(kitty_id);

			Self::deposit_event(Event::KittyBought {who, kitty_id});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_100 + T::DbWeight::get().writes(1).ref_time())]
		pub fn sale(origin: OriginFor<T>, kitty_id: u32)-> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			let price = T::KittyPrice::get();
			T::Currency::transfer(&who, &Self::get_account_id(), price, ExistenceRequirement::KeepAlive)?;

			KittyOnSale::<T>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale {who, kitty_id});
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

		fn get_account_id()-> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
