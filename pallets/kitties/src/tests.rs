use super::*;
use crate::{mock::*, Error, Event, Kitties, KittyId, NextKittyId};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
		assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
		assert_eq!(KittiesModule::kitty_parent(kitty_id), None);

		NextKittyId::<Test>::set(KittyId::max_value());
		assert_noop!(
			KittiesModule::create_kitty((RuntimeOrigin::signed(account_id))),
			Error::<Test>::InvalidKittyId
		);

		// 判断最后一次事件是否是 CreateKittyEvent
		System::assert_last_event(
			Event::CreateKittyEvent {
				who: 1,
				kitty_id,
				kitty: KittiesModule::kitties(kitty_id).unwrap(),
			}
			.into(),
		);
	})
}

#[test]
fn it_works_for_breed() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
			Error::<Test>::SameKittyId
		);
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));

		assert_eq!(KittiesModule::kitties(0).is_some(), true);
		assert_eq!(KittiesModule::kitties(1).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(0), Some(account_id));
		assert_eq!(KittiesModule::kitty_owner(1), Some(account_id));
		assert_eq!(KittiesModule::kitty_parent(0), None);
		assert_eq!(KittiesModule::kitty_parent(1), None);

		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), 2, kitty_id),
			Error::<Test>::InvalidKittyId
		);
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, 2),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), 0, 1));
		// 判断最后一次事件是否是 KittyBreed
		System::assert_last_event(
			Event::KittyBreed { who: 1, kitty_id: 2, kitty: KittiesModule::kitties(2).unwrap() }
				.into(),
		);

		assert_eq!(KittiesModule::kitties(2).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(2), Some(account_id));
		assert_eq!(KittiesModule::kitty_parent(2), Some((0, 1)));
	})
}

#[test]
fn it_works_for_transfered() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;

		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id),
			Error::<Test>::InvalidKittyId
		);
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(account_id)));
		assert_eq!(KittiesModule::kitty_owner(0), Some(account_id));

		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id),
			Error::<Test>::NotOwner
		);

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id));
		// 判断最后一次事件是否是 KittyBreed
		System::assert_last_event(Event::KittyTransfered { who: 1, recipient, kitty_id }.into());

		assert_eq!(KittiesModule::kitties(0).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(0), Some(recipient));
		assert_eq!(KittiesModule::kitty_parent(0), None);
	})
}
