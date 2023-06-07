use super::*;
use crate::{mock::*, Error, Event, Kitties, KittyId, NextKittyId};
use frame_support::{assert_noop, assert_ok};
use frame_support::dispatch::DispatchResult;

fn create_kitty(account_id: u64) -> DispatchResult {
	KittiesModule::create_kitty(RuntimeOrigin::signed(account_id), *b"12345678")
}

fn bread(account_id: u64, kitty_id1: u32, kitty_id2: u32) -> DispatchResult {
	KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id1, kitty_id2, *b"12345678")
}


#[test]
fn it_works_for_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
		assert_ok!(create_kitty(account_id));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
		assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
		assert_eq!(KittiesModule::kitty_parent(kitty_id), None);

		NextKittyId::<Test>::set(KittyId::max_value());
		assert_noop!(
			create_kitty(account_id),
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
			bread(account_id, kitty_id, kitty_id),
			Error::<Test>::SameKittyId
		);
		assert_ok!(create_kitty(account_id));
		assert_ok!(create_kitty(account_id));

		assert_eq!(KittiesModule::kitties(0).is_some(), true);
		assert_eq!(KittiesModule::kitties(1).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(0), Some(account_id));
		assert_eq!(KittiesModule::kitty_owner(1), Some(account_id));
		assert_eq!(KittiesModule::kitty_parent(0), None);
		assert_eq!(KittiesModule::kitty_parent(1), None);

		assert_noop!(
			bread(account_id, 2, kitty_id),
			Error::<Test>::InvalidKittyId
		);
		assert_noop!(
			bread(account_id, kitty_id, 2),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(bread(account_id, 0, 1));
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
		assert_ok!(create_kitty(account_id));
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

#[test]
fn it_works_for_buy() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let next_account_id = 2;

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(create_kitty(account_id));

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOwned
		);

		assert_ok!(create_kitty(next_account_id));

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), 1),
			Error::<Test>::NotOnSale
		);

		// 将kitty置为sale状态
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
		// 购买kitty
		assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(next_account_id), kitty_id));
		// 判断购买后kitty的拥有者是否为购买的人
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(next_account_id));
		// 判断kitty是否从待销售状态中移除
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id), None);
	})
}

#[test]
fn it_works_for_sale() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let next_account_id = 2;

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);
		assert_ok!(create_kitty(account_id));
		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(next_account_id), kitty_id),
			Error::<Test>::NotOwner
		);

		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
		// kitty置为sale状态
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id), Some(()));

		// 断言重复销售
		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOnSale
		);
	})
}
