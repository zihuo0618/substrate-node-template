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

use frame_system::{
    offchain::{
        AppCrypto, CreateSignedTransaction, SendUnsignedTransaction,
        SignedPayload, Signer, SigningTypes,
    },
};

use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");

pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
    for OcwAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::inherent::Vec;
    use frame_support::log;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::sp_std::vec;
    use sp_io::offchain_index;
    use sp_runtime::{
        offchain::{
            http, Duration,
        },
    };
    use scale_info::prelude::string::String;
    use sp_std::borrow::ToOwned;



    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct Payload<Public> {
        areacode: String,
        public: Public,
    }

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Response {
        success: String,
        result: ResultSt,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct ResultSt {
        count: String,
        lists: Vec<List>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct List {
        areaid: String,
        postcode: String,
        areacode: String,
        areanm: String,
        simcall: String,
    }

    impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
        fn public(&self) -> T::Public {
            self.public.clone()
        }
    }


    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    }

    // The pallet's runtime storage items.
    // https://docs.substrate.io/main-docs/build/runtime-storage/
    #[pallet::storage]
    #[pallet::getter(fn something)]
    // Learn more about declaring storage items:
    // https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
    pub type Something<T> = StorageValue<_, u32>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/main-docs/build/events-errors/
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored { something: u32, who: T::AccountId },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/main-docs/build/origins/
            let who = ensure_signed(origin)?;

            // Update storage.
            <Something<T>>::put(something);

            // Emit an event.
            Self::deposit_event(Event::SomethingStored { something, who });
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1).ref_time())]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Read a value from storage.
            match <Something<T>>::get() {
                // Return an error if the value has not been set.
                None => return Err(Error::<T>::NoneValue.into()),
                Some(old) => {
                    // Increment the value read from storage; will error in the event of overflow.
                    let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
                    // Update the value in storage with the incremented result.
                    <Something<T>>::put(new);
                    Ok(())
                }
            }
        }

        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn unsigned_extrinsic_with_signed_payload(origin: OriginFor<T>, payload: Payload<T::Public>, _signature: T::Signature) -> DispatchResult {
            ensure_none(origin)?;
            let key = Self::derive_key(frame_system::Pallet::<T>::block_number());
            log::info!("OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}", payload.areacode);
            let hex_string = hex::encode(key.clone());
            log::info!("OCW key: {}", hex_string);
            offchain_index::set(&key, payload.areacode.as_bytes());
            Ok(())
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
            let valid_tx = |provide| ValidTransaction::with_tag_prefix("my-pallet")
                .priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
                .and_provides([&provide])
                .longevity(3)
                .propagate(true)
                .build();

            match call {
                Call::unsigned_extrinsic_with_signed_payload {
                    ref payload,
                    ref signature
                } => {
                    if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
                        return InvalidTransaction::BadProof.into();
                    }
                    valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            log::info!("OCW ==> Hello World from offchain workers!: {:?}", block_number);
            // Retrieve the signer to sign the payload
            let signer = Signer::<T, T::AuthorityId>::any_account();
            let result = Pallet::<T>::fetch_info();
            if let Ok(ret) = result {
                // `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
                //	 The returned result means:
                //	 - `None`: no account is available for sending transaction
                //	 - `Some((account, Ok(())))`: transaction is successfully sent
                //	 - `Some((account, Err(())))`: error occurred when sending the transaction
                if let Some((_, res)) = signer.send_unsigned_transaction(
                    // this line is to prepare and return payload
                    |acct| Payload { areacode: ret.result.lists.get(0).map_or("11111".to_owned(), |v| v.areacode.clone()), public: acct.public.clone() },
                    |payload, signature| Call::unsigned_extrinsic_with_signed_payload { payload, signature },
                ) {
                    match res {
                        Ok(()) => { log::info!("OCW ==> unsigned tx with signed payload successfully sent."); }
                        Err(()) => { log::error!("OCW ==> sending unsigned tx with signed payload failed."); }
                    };
                } else {
                    // The case of `None`: no account is available for sending
                    log::error!("OCW ==> No local account available");
                }
            }

        }
    }

    impl<T: Config> Pallet<T> {
        #[deny(clippy::clone_double_ref)]
        fn derive_key(block_number: T::BlockNumber) -> Vec<u8> {
            block_number.using_encoded(|encoded_bn| {
                b"node-template::storage::"
                    .iter()
                    .chain(encoded_bn)
                    .copied()
                    .collect::<Vec<u8>>()
            })
        }

        fn fetch_info() -> Result<Response, http::Error> {
            // prepare for send request
            let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
            let request =
                http::Request::get("http://api.k780.com/?app=life.postcode&postcode=528400&appkey=10003&sign=b59bc3ef6191eb9f747dd4e83c99f2a4&format=json");
            let pending = request
                .add_header("User-Agent", "Substrate-Offchain-Worker")
                .deadline(deadline).send().map_err(|_| http::Error::IoError)?;
            let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
            if response.code != 200 {
                log::warn!("Unexpected status code: {}", response.code);
                return Err(http::Error::Unknown);
            }
            let body = response.body().collect::<Vec<u8>>();
            let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
                log::warn!("No UTF8 body");
                http::Error::Unknown
            })?;

            // parse the response str
            let response: Response =
                serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

            Ok(response)
        }
    }
}
