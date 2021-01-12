use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        assert_ok!(TemplateModule::create_claim(Origin::signed(1), claim.clone()));

        assert_eq!(
            Proofs::<Test>::get(&claim),
            (1, frame_system::Module::<Test>::block_number())
        );
    })
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = TemplateModule::create_claim(Origin::signed(1), claim.clone());

        assert_noop!(
            TemplateModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );
    })
}

#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = TemplateModule::create_claim(Origin::signed(1), claim.clone());

        assert_ok!(TemplateModule::revoke_claim(Origin::signed(1), claim.clone()));
    })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(
            TemplateModule::revoke_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimNotExist
        );
    })
}

#[test]
fn transfer_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = TemplateModule::create_claim(Origin::signed(1), claim.clone());

        let dest: u64 = 10;
        assert_ok!(TemplateModule::transfer_claim(Origin::signed(1), claim.clone(), dest));
    })
}

#[test]
fn transfer_claim_failed_when_claim_no_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = TemplateModule::create_claim(Origin::signed(1), claim.clone());

        let no_exist_claim = vec![2, 3];

        let dest: u64 = 10;
        assert_noop!(
            TemplateModule::transfer_claim(Origin::signed(1), no_exist_claim.clone(), dest),
            Error::<Test>::ClaimNotExist
        );
    })
}

#[test]
fn transfer_claim_too_long() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

        assert_noop!(
            TemplateModule::create_claim(Origin::signed(1), claim),
            Error::<Test>::ClaimTooLong
        );
    })
}