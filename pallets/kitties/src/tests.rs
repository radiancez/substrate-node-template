use crate::{mock::*, Error, Event, KittyId, NextKittyId};
use frame_support::{assert_noop, assert_ok};

const ACCOUNT_ID_1: u64 = 1;
const ACCOUNT_ID_2: u64 = 2;
const KITTY_ID_0: KittyId = 0;

#[test]
fn create_kitty() {
	new_test_ext().execute_with(|| {
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);

		// 检查初始状态
		assert_eq!(PalletKitties::next_kitty_id(), KITTY_ID_0);

		// create kitty
		assert_ok!(PalletKitties::create_kitty(signer.clone()));
		assert_eq!(PalletKitties::next_kitty_id(), KITTY_ID_0 + 1);
		assert_eq!(PalletKitties::kitties(KITTY_ID_0).is_some(), true);
		assert_eq!(PalletKitties::kitty_owner(KITTY_ID_0), Some(ACCOUNT_ID_1));
		assert_eq!(PalletKitties::kitty_parents(KITTY_ID_0), None);
		System::assert_last_event(
			Event::KittyCreated {
				account: ACCOUNT_ID_1,
				kitty_id: KITTY_ID_0,
				kitty: PalletKitties::kitties(KITTY_ID_0).unwrap(),
			}
			.into(),
		);

		// KittyId 溢出
		NextKittyId::<Test>::set(KittyId::max_value());
		assert_noop!(PalletKitties::create_kitty(signer), Error::<Test>::KittyIdOverflow);
	});
}

#[test]
fn bred_kitty() {
	new_test_ext().execute_with(|| {
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);

		let parent_id_0 = KITTY_ID_0;
		let parent_id_1 = KITTY_ID_0 + 1;
		let child_id = KITTY_ID_0 + 2;

		// parent 相同
		assert_noop!(
			PalletKitties::bred_kitty(signer.clone(), parent_id_0, parent_id_0),
			Error::<Test>::SameParentKittyId
		);
		// parent 不存在
		assert_noop!(
			PalletKitties::bred_kitty(signer.clone(), parent_id_0, parent_id_1),
			Error::<Test>::KittyNotExist
		);

		// 创建两只Kitty作为parent
		assert_ok!(PalletKitties::create_kitty(signer.clone()));
		assert_ok!(PalletKitties::create_kitty(signer.clone()));
		assert_eq!(PalletKitties::next_kitty_id(), child_id);

		// bred kitty
		assert_ok!(PalletKitties::bred_kitty(signer, parent_id_0, parent_id_1));
		assert_eq!(PalletKitties::next_kitty_id(), child_id + 1);
		assert_eq!(PalletKitties::kitties(child_id).is_some(), true);
		assert_eq!(PalletKitties::kitty_owner(child_id), Some(ACCOUNT_ID_1));
		assert_eq!(PalletKitties::kitty_parents(child_id), Some((parent_id_0, parent_id_1)));
		System::assert_last_event(
			Event::KittyBred {
				account: ACCOUNT_ID_1,
				kitty_id: child_id,
				kitty: PalletKitties::kitties(child_id).unwrap(),
			}
			.into(),
		);
	});
}

#[test]
fn transfer_kitty() {
	new_test_ext().execute_with(|| {
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);
		let signer_2 = RuntimeOrigin::signed(ACCOUNT_ID_2);

		// create kitty
		assert_ok!(PalletKitties::create_kitty(signer.clone()));
		assert_eq!(PalletKitties::kitty_owner(KITTY_ID_0), Some(ACCOUNT_ID_1));

		// 非ower进行transfer
		assert_noop!(
			PalletKitties::transfer_kitty(signer_2.clone(), ACCOUNT_ID_1, KITTY_ID_0),
			Error::<Test>::NotKittyOwner
		);

		// transfer 1 -> 2
		assert_ok!(PalletKitties::transfer_kitty(signer, ACCOUNT_ID_2, KITTY_ID_0));
		assert_eq!(PalletKitties::kitty_owner(KITTY_ID_0), Some(ACCOUNT_ID_2));
		System::assert_last_event(
			Event::KittyTransferred {
				sender: ACCOUNT_ID_1,
				recipient: ACCOUNT_ID_2,
				kitty_id: KITTY_ID_0,
			}
			.into(),
		);

		// transfer 2 -> 1
		assert_ok!(PalletKitties::transfer_kitty(signer_2, ACCOUNT_ID_1, KITTY_ID_0));
		assert_eq!(PalletKitties::kitty_owner(KITTY_ID_0), Some(ACCOUNT_ID_1));
		System::assert_last_event(
			Event::KittyTransferred {
				sender: ACCOUNT_ID_2,
				recipient: ACCOUNT_ID_1,
				kitty_id: KITTY_ID_0,
			}
			.into(),
		);
	});
}
