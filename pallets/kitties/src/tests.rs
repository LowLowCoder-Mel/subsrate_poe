use super::*;
use sp_core::H256;
use frame_support::{impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop,
                    traits::{ OnFinalize, OnInitialize, Currency, LockIdentifier, LockableCurrency, ExistenceRequirement, WithdrawReason, WithdrawReasons },
};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup, AtLeast32Bit, Bounded, Member}, testing::Header, Perbill,
};
use frame_system as system;

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

type Randomness = pallet_randomness_collective_flip::Module<Test>;

impl Trait for Test {
    type Event = ();
    type KittyIndex = u64;
    type Currency = pallet_balances::Module<Self>;
    type Randomness = Randomness;
}

pub type Kitties = Module<Test>;
pub type System = frame_system::Module<Test>;

fn run_to_block(n: u64) {
    while System::block_number() < n {
        Kitties::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Kitties::on_initialize(System::block_number());
    }
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

/// 创建kitty
#[test]
fn owned_kitties_can_append_values() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(Kitties::create(Origin::signed(1)), Ok(()))
    })
}

// /// transfer kitty
// #[test]
// fn transfer_kitties() {
//     new_test_ext().execute_with(|| {
//         run_to_block(10);
//         assert_ok!(Kitties::create(Origin::signed(1)));
//         let id = Kitties::kitties_count();
//         assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id-1));
//         assert_noop!(
//                 Kitties::transfer(Origin::signed(1), 2, id-1),
//                 Error::<Test>::NotKittyOwner
//                 );
//     })
// }

#[test]
fn deposit_event_should_work() {
    new_test_ext().execute_with(|| {
        System::initialize(
            &1,
            &[0u8; 32].into(),
            &Default::default(),
            InitKind::Full,
        );
        System::note_finished_extrinsics();
        System::deposit_event(SysEvent::CodeUpdated);
        System::finalize();
        assert_eq!(
            System::events(),
            vec![
                EventRecord {
                    phase: Phase::Finalization,
                    event: SysEvent::CodeUpdated,
                    topics: vec![],
                }
            ]
        );

        System::initialize(
            &2,
            &[0u8; 32].into(),
            &Default::default(),
            InitKind::Full,
        );
        System::deposit_event(SysEvent::NewAccount(32));
        System::note_finished_initialize();
        System::deposit_event(SysEvent::KilledAccount(42));
        System::note_applied_extrinsic(&Ok(().into()), Default::default());
        System::note_applied_extrinsic(
            &Err(DispatchError::BadOrigin.into()),
            Default::default()
        );
        System::note_finished_extrinsics();
        System::deposit_event(SysEvent::NewAccount(3));
        System::finalize();
        assert_eq!(
            System::events(),
            vec![
                EventRecord {
                    phase: Phase::Initialization,
                    event: SysEvent::NewAccount(32),
                    topics: vec![],
                },
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: SysEvent::KilledAccount(42),
                    topics: vec![]
                },
                EventRecord {
                    phase: Phase::ApplyExtrinsic(0),
                    event: SysEvent::ExtrinsicSuccess(Default::default()),
                    topics: vec![]
                },
                EventRecord {
                    phase: Phase::ApplyExtrinsic(1),
                    event: SysEvent::ExtrinsicFailed(
                        DispatchError::BadOrigin.into(),
                        Default::default()
                    ),
                    topics: vec![]
                },
                EventRecord {
                    phase: Phase::Finalization,
                    event: SysEvent::NewAccount(3),
                    topics: vec![]
                },
            ]
        );
    });
}