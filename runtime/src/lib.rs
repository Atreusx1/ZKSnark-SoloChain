// ------------------------
// Import Necessary Crates
// ------------------------
//runtime/src/lib.rs

use frame_support::{
    dispatch::DispatchResult, 
    parameter_types, 
    traits::{ConstU32, BuildGenesisConfig}, 
    weights::Weight, 
    construct_runtime, 
    pallet_prelude::*,
};
use sp_version::{RuntimeVersion, create_runtime_str};
use sp_runtime::{
    traits::{Verify, Hash as HashT, IdentifyAccount}, 
    MultiAddress, MultiSignature, RuntimeDebug, generic,
};
use sp_core::H256;
use pallet_timestamp::{Pallet as TimestampPallet, Call as TimestampCall, Inherent};
use pallet_aura::{Pallet as AuraPallet, Call as AuraCall, Config as AuraConfig};
use pallet_grandpa::{Pallet as GrandpaPallet, Call as GrandpaCall, Storage as GrandpaStorage, Config as GrandpaConfig, Event as GrandpaEvent};
use pallet_balances::{Pallet as BalancesPallet, Call as BalancesCall, Storage as BalancesStorage, Config as BalancesConfig, Event as BalancesEvent};
use pallet_transaction_payment::{Pallet as TransactionPaymentPallet, Storage as TransactionPaymentStorage};
use pallet_sudo::{Pallet as SudoPallet, Call as SudoCall, Storage as SudoStorage, Config as SudoConfig, Event as SudoEvent};
use zk_snarks_lib::{generate_zk_proof, verify_zk_proof, utils::random_witness};
use pallet_zksnark;

parameter_types! {
    pub const MerkleTreeDepth: u32 = 20;
    pub const MaxBatchSize: u32 = 4;
}

impl pallet_zksnark::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MerkleTreeDepth = MerkleTreeDepth;
    type MaxBatchSize = MaxBatchSize;
}
use sp_std::{format, prelude::*, vec::Vec};

// ------------------------
// Type Definitions
// ------------------------

pub type Balance = u128;
pub type BlockNumber = u32;
pub type Nonce = u32;
pub type Hash = sp_core::H256;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Address = MultiAddress<AccountId, ()>;

// ------------------------
// Opaque Types
// ------------------------

pub mod opaque {
    use super::*;
    use sp_runtime::traits::Hash as HashT;

    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;
    pub type Hash = <BlakeTwo256 as HashT>::Output;
}

// ------------------------
// Runtime Version
// ------------------------

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("solochain-template-runtime"),
    impl_name: create_runtime_str!("solochain-template-runtime"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    transaction_version: 1,
    state_version: 1,
    apis: &[], // Add any runtime APIs here if you have them
};

// ------------------------
// Session Keys
// ------------------------

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
    }
}

// ------------------------
// Construct Runtime
// ------------------------

construct_runtime!(
    pub enum Runtime where
        Block = opaque::Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // Core Pallets
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Inherent},
        Aura: pallet_aura::{Pallet, Call, Storage, Config<T>},
        Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
        Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},

        // Custom Pallet
        ZkSnark: pallet_zksnark,
    }
);

// ------------------------
// Signed Extra
// ------------------------

type SignedExtra = (
    system::CheckVersion<Runtime>,
    system::CheckGenesis<Runtime>,
    system::CheckEra<Runtime>,
    system::CheckNonce<Runtime>,
    system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

// ------------------------
// Define Unchecked Extrinsic and Block Types
// ------------------------

pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
pub type Block = generic::Block<opaque::Header, UncheckedExtrinsic>;

// ------------------------
// Implement Config Traits for Pallets
// ------------------------

// System Config
impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = Nonce;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
    type Header = opaque::Header;
    type Event = Event;
    type BlockHashCount = ConstU32<2400>;
    type Version = VERSION;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU32<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

// Timestamp Config
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU32<{ MILLI_SECS_PER_BLOCK / 2 }>;
    type WeightInfo = ();
}

// Aura Config
impl pallet_aura::Config for Runtime {
    type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<16>;
}

// Grandpa Config
impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type KeyOwnerProof = sp_consensus_grandpa::OpaqueKeyOwnershipProof;
    type KeyProvider = Grandpa;
    type HandleEquivocation = ();
    type WeightInfo = ();
}

// Balances Config
impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU32<1_000_000_000>;
    type AccountStore = System;
    type WeightInfo = ();
}

// Transaction Payment Config
impl pallet_transaction_payment::Config for Runtime {}

// Sudo Config
impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

impl pallet_zksnark::Config for Runtime {
    type RuntimeEvent = RuntimeEvent; // Ensure this is consistent with your runtime event type.
    type VerifyingKey = VerifyingKeyStorage<Self>;
    type WeightInfo = ();
}


// ------------------------
// Verifying Key Storage
// ------------------------

pub struct VerifyingKeyStorage<T>(sp_std::marker::PhantomData<T>);

impl<T: frame_system::Config> pallet_zksnark::VerifyingKeyStorage<T> for VerifyingKeyStorage<T> {
    type Key = Vec<u8>;

    fn put(key: Vec<u8>) -> DispatchResult {
        <VerifyingKey<T>>::put(key);
        Ok(())
    }

    fn get() -> Option<Vec<u8>> {
        <VerifyingKey<T>>::get()
    }
}

// ------------------------
// Define Storage Item for Verifying Key
// ------------------------

#[frame_support::pallet]
pub mod pallet_zksnark {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn verifying_key)]
    pub type VerifyingKey<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type VerifyingKey: VerifyingKeyStorage<Self>;
        // Add other associated types as needed
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        VerifyingKeyUpdated,
        // Add other events as needed
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn update_verifying_key(origin: OriginFor<T>, key: Vec<u8>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            <VerifyingKey<T>>::put(key);
            Self::deposit_event(Event::VerifyingKeyUpdated);
            Ok(())
        }

        // Add other dispatchable functions as needed
    }

    // Implement GenesisBuild for ZkSnark Pallet
    impl<T: Config> BuildGenesisConfig<T> for GenesisConfig<T> {
        fn build(&self) {
            if !self.verifying_key.is_empty() {
                <VerifyingKey<T>>::put(self.verifying_key.clone());
                Pallet::<T>::deposit_event(Event::VerifyingKeyUpdated);
            }
        }
    }
}

// ------------------------
// Genesis Configuration
// ------------------------

#[pallet::genesis_config]
pub struct BuildGenesisConfig<T: Config> {
    pub verifying_key: Vec<u8>,
}

impl<T: Config> Default for BuildGenesisConfig<T> {
    fn default() -> Self {
        GenesisConfig {
            verifying_key: Vec::new(),
        }
    }
}

