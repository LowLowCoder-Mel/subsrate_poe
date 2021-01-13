use crate::{ Event, Error, mock::* };
use frame_support::{
    assert_noop, assert_ok,
    traits::{ OnFinalize, OnInitialize }
};
use frame_system::{ EventRecord, Phase };

fn run_to_block(n: u64) {
    while System::block_number() < n {
        Kitties::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Kitties::on_initialize(System::block_number());
    }
}

// 测试创建一个 Kitty
#[test]
fn create_kitty_event_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Kitties::create(Origin::signed(1)));

        assert_eq!(
            System::events(),
            vec![EventRecord {
                phase: Phase::Initialization,
                event: TestEvent::kitties_event( Event::<Test>::Created(1u64 , 0)),
                topics: vec![],
            }]
        );
    })
}

// 测试转让 Kitty 成功
#[test]
fn transfer_kitty_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);

        let _ = Kitties::create(Origin::signed(1));

        assert_ok!(Kitties::transfer(Origin::signed(1), 2, 0));
    })
}

// 测试转让 Kitty 失败，因为 kitty 不存在
#[test]
fn transfer_kitty_failed_when_not_exists(){
    new_test_ext().execute_with(|| {

        assert_noop!(Kitties::transfer(Origin::signed(1), 2, 0), Error::<Test>::RequireOwner);

    })
}

#[test]
fn transfer_kitties_work() {
    new_test_ext().execute_with(|| {
        run_to_block(10);

        assert_ok!(Kitties::create(Origin::signed(1)));
        let id = Kitties::kitties_count();
        assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id - 1));
        assert_noop!(
                Kitties::transfer(Origin::signed(1), 2, id - 1),
                Error::<Test>::RequireOwner);
    })
}

#[test]
fn breed_kitty_work() {
    new_test_ext().execute_with(|| {
        run_to_block(10);

        let _ = Kitties::create(Origin::signed(1));
        let _ = Kitties::create(Origin::signed(1));

        assert_ok!(Kitties::breed(Origin::signed(1), 0, 1));
    })
}

#[test]
fn breed_kitty_fail_when_parent_same() {
    new_test_ext().execute_with(|| {
        run_to_block(10);

        let _ = Kitties::create(Origin::signed(1));

        assert_noop!(Kitties::breed(Origin::signed(1), 0, 0), Error::<Test>::RequireDifferentParent);

    })
}

#[test]
fn breed_kitty_fail_when_parent_not_exist() {
    new_test_ext().execute_with(|| {
        assert_noop!(Kitties::breed(Origin::signed(1), 0, 0), Error::<Test>::InvalidKittyId);
    })
}

#[test]
fn breed_kitty_fail_when_not_owner() {
    new_test_ext().execute_with(|| {
        run_to_block(10);

        let _ = Kitties::create(Origin::signed(1));
        let _ = Kitties::create(Origin::signed(1));

        assert_noop!(Kitties::breed(Origin::signed(2), 0, 1), Error::<Test>::RequireOwner);
    })
}