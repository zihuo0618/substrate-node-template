use frame_system::pallet_prelude::*;
use frame_support::pallet_prelude::*;
use frame_support::Blake2_128Concat;
use frame_support::migration::storage_key_iter;
use frame_support::weights::Weight;
use frame_support::StoragePrefixedMap;
use sp_runtime::print;

use crate::{Config, Kitties, Kitty, KittyId, Pallet};

#[derive(
Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct OldKitty_V0(pub [u8; 16]);

#[derive(
Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct OldKitty_V1 {
    pub dna: [u8; 16],
    pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
    let on_chain_version = Pallet::<T>::on_chain_storage_version();
    let current_version = Pallet::<T>::current_storage_version();

    let module = Kitties::<T>::module_prefix();
    let item = Kitties::<T>::storage_prefix();
    log::info!("chain version {:?}, current version {:?}", on_chain_version, current_version);
    log::info!("{:?}, {:?}", module, item);

    if on_chain_version == 0 && current_version == 2 {
        for (index, kitty) in storage_key_iter::<KittyId, OldKitty_V0, Blake2_128Concat>(module, item).drain() {
            let new_kitty = Kitty {
                dna: kitty.0,
                name: *b"abcdfdff",
            };
            Kitties::<T>::insert(&index, &new_kitty);
        }
    } else if on_chain_version == 1 && current_version == 2 {
        for (index, kitty) in storage_key_iter::<KittyId, OldKitty_V1, Blake2_128Concat>(module, item).drain() {
            let mut new_name = [0u8; 8];
            new_name[..4].copy_from_slice(&kitty.name);
            let new_kitty = Kitty {
                dna: kitty.dna,
                name: new_name,
            };
            Kitties::<T>::insert(&index, &new_kitty);
        }
    }
    Weight::zero()
}