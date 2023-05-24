use super::*;
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn test_create_claim() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![1, 2, 3];
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
        assert_eq!(Proofs::<Test>::get(BoundedVec::try_from(claim.clone()).unwrap()), Some((1, frame_system::Pallet::<Test>::block_number())));
    });
}

#[test]
fn test_create_claim_failed() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![1, 2, 3];
        assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()));
        assert_noop!(PoeModule::create_claim(RuntimeOrigin::signed(1),
                     vec![1u8 ,2u8,3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8]),
                     Error::<Test>::ClaimTooLong);
        assert_noop!(PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone()), Error::<Test>::ProofAlreadyExist);
    });
}

#[test]
fn test_remove_claim_ok() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![1, 2, 3];
        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());
        assert_ok!(PoeModule::remove_claim(RuntimeOrigin::signed(1), claim.clone()));
    });
}

#[test]
fn test_remove_claim_failed() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![1, 2, 3];
        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());
        assert_noop!(PoeModule::remove_claim(RuntimeOrigin::signed(2), claim.clone()),
                     Error::<Test>::NotClaimOwner);
        assert_noop!(PoeModule::remove_claim(RuntimeOrigin::signed(1), vec![2u8,3u8, 4u8]),
                     Error::<Test>::ClaimNotExist);
        assert_noop!(PoeModule::remove_claim(RuntimeOrigin::signed(1),
                vec![1u8 ,2u8,3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8]),
                Error::<Test>::ClaimTooLong);
    });
}

#[test]
fn test_transfer_claim_ok() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![1, 2, 3];
        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());
        assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(1), claim.clone(), 2));
    });
}

#[test]
fn test_transfer_claim_failed() {
    new_test_ext().execute_with(|| {
        let claim: Vec<u8> = vec![1, 2, 3];
        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());
        assert_noop!(PoeModule::transfer_claim(RuntimeOrigin::signed(2), claim.clone(), 3),
                Error::<Test>::NotClaimOwner);
        assert_noop!(PoeModule::transfer_claim(RuntimeOrigin::signed(2),
                vec![2u8,3u8,4u8],
                3),
                Error::<Test>::ClaimNotExist);
    });
}
