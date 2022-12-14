use change_set::{diff::OptionDiff, do_action};
use frame_support::{
	assert_noop, assert_ok,
	sp_runtime::{
		traits::{GetNodeBlockType, GetRuntimeBlockType},
		AccountId32,
	},
	traits::{Hooks, OriginTrait},
};
use frame_system::{pallet_prelude::OriginFor, Config as SystemConfig, Pallet as System};

use crate::{Config, Event, Pallet, Something};

const ALICE: AccountId32 = AccountId32::new([0; 32]);

pub trait ExamplePalletRuntimeBounds:
	SystemConfig<
		BlockNumber = Self::SystemBlockNumber,
		RuntimeEvent = Self::SystemRuntimeEvent,
		AccountId = Self::SystemAccountId,
		RuntimeOrigin = Self::SystemRuntimeOrigin,
	> + Config
{
	type SystemBlockNumber: From<u64>;
	type SystemRuntimeEvent: From<Event<Self>>;
	type SystemAccountId: From<AccountId32>;
	type SystemRuntimeOrigin: OriginTrait<AccountId = Self::SystemAccountId>;
}

impl<T> ExamplePalletRuntimeBounds for T
where
	T: SystemConfig + Config,
	<T as SystemConfig>::BlockNumber: From<u64>,
	<T as SystemConfig>::RuntimeEvent: From<Event<Self>>,
	<T as SystemConfig>::AccountId: From<AccountId32>,
	<T as SystemConfig>::RuntimeOrigin: OriginTrait<AccountId = <Self as SystemConfig>::AccountId>,
{
	type SystemAccountId = <T as SystemConfig>::AccountId;
	type SystemBlockNumber = <T as SystemConfig>::BlockNumber;
	type SystemRuntimeEvent = <T as SystemConfig>::RuntimeEvent;
	type SystemRuntimeOrigin = <T as SystemConfig>::RuntimeOrigin;
}

pub fn it_works_for_default_value<T>()
where
	T: ExamplePalletRuntimeBounds, /* + GetNodeBlockType
	                                * + GetRuntimeBlockType
	                                * + sp_block_builder::BlockBuilder<<T as
	                                * GetNodeBlockType>::NodeBlock>, */
{
	// Go past genesis block so events get deposited
	System::<T>::set_block_number(1.into());

	// System::on_initialize
	// Dispatch a signed extrinsic.
	do_action::<(Something<T>, ()), _, _>(|| {
		Pallet::<T>::do_something(OriginFor::<T>::signed(ALICE.into()), 42).unwrap();
		System::<T>::assert_last_event(
			Event::SomethingStored { something: 42, who: ALICE.into() }.into(),
		);
	})
	.check_storage::<Something<T>, _>(OptionDiff::WasNoneNowSome(42))
	.assert_storage_changes();

	// Assert that the correct event was deposited
}

pub fn correct_error_for_none_value<T>()
where
	T: ExamplePalletRuntimeBounds,
{
	// Ensure the expected error is thrown when no value is present.
	assert_noop!(
		Pallet::<T>::cause_error(OriginFor::<T>::signed(ALICE.into())),
		crate::Error::<T>::NoneValue
	);
}
