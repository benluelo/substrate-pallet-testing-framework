use core::{fmt::Debug, iter, marker::PhantomData};

use hlist::{AssertDiffHList, Concat, Concatenated};

use crate::{
	check_storage::CheckStorage,
	diff::{Diff, Diffable},
	hlist::{
		AssertionOutputOf, ExpectedChangesOf, Find, HList, HZippable, PalletStorageHList, Zipped,
	},
};

pub mod check_storage;
pub mod diff;
pub mod hlist;

// name is bikeshedding lol
pub struct AssertableDiffableStorageAction<
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	F: FnOnce(),
> {
	f: F,
	storage_checker: StorageChecker<UncheckedStorages, CheckedStorages>,
}

// a better name is welcome
pub fn do_action<UncheckedStorages: PalletStorageHList, F: FnOnce()>(
	f: F,
) -> AssertableDiffableStorageAction<UncheckedStorages, (), F> {
	AssertableDiffableStorageAction {
		f,
		storage_checker: StorageChecker { expected_changes: (), _marker: PhantomData },
	}
}

impl<CheckedStorages, UncheckedStorages, F>
	AssertableDiffableStorageAction<UncheckedStorages, CheckedStorages, F>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	F: FnOnce(),
{
	#[must_use = "check_storage does nothing on it's own, assert_storage_changes must be called to actually do the checks"]
	pub fn check_storage<T: CheckStorage, Index>(
		self,
		t_value: <T::Value as Diffable>::ChangeSet,
	) -> AssertableDiffableStorageAction<
		<UncheckedStorages as Find<T, Index>>::Remainder,
		(T, CheckedStorages),
		F,
	>
	where
		<T::Value as Diffable>::ChangeSet: Debug,
		UncheckedStorages: Find<T, Index>,
		<UncheckedStorages as Find<T, Index>>::Remainder: PalletStorageHList,
	{
		AssertableDiffableStorageAction {
			f: self.f,
			storage_checker: self.storage_checker.check_storage(t_value),
		}
	}
}

impl<CheckedStorages, UncheckedStorages, F>
	AssertableDiffableStorageAction<UncheckedStorages, CheckedStorages, F>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	// <CheckedStorages as PalletStorageHList>::CurrentValue:
	// 	DiffableHList<ChangeSet = ExpectedChangesOf<CheckedStorages>>,
	UncheckedStorages::NamesOutput: HZippable<AssertionOutputOf<UncheckedStorages>>,
	CheckedStorages::NamesOutput: HZippable<AssertionOutputOf<CheckedStorages>>,
	Zipped<UncheckedStorages::NamesOutput, AssertionOutputOf<UncheckedStorages>>:
		Concat<Zipped<CheckedStorages::NamesOutput, AssertionOutputOf<CheckedStorages>>>,
	Concatenated<
		Zipped<UncheckedStorages::NamesOutput, AssertionOutputOf<UncheckedStorages>>,
		Zipped<CheckedStorages::NamesOutput, AssertionOutputOf<CheckedStorages>>,
	>: HListIntoIterator<(
		// TODO(benluelo): Use a better type here
		// (pallet prefix, storage prefix)
		(&'static str, &'static str),
		Option<String>,
	)>,
	F: FnOnce(),
{
	pub fn assert_storage_changes(self) {
		let maybe_errors = self
			.storage_checker
			.check(self.f)
			.into_iter()
			.filter_map(|(storage, maybe_error)| maybe_error.map(|error| (storage, error)))
			.map(|((k1, k2), v)| format!("ERROR at storage {k1}/{k2}: {v}"))
			.collect::<Vec<_>>();

		if !maybe_errors.is_empty() {
			panic!("{}", maybe_errors.join("\n\n"));
		}
	}
}

// REVIEW(benluelo): Should this be generic? Or should it have an `Item` associated type?
pub trait HListIntoIterator<T> {
	type Iterator: Iterator<Item = T>;

	fn into_iter(self) -> Self::Iterator;
}

impl<T> HListIntoIterator<T> for () {
	type Iterator = iter::Empty<T>;

	fn into_iter(self) -> Self::Iterator {
		iter::empty()
	}
}

impl<Head, Tail> HListIntoIterator<Head> for (Head, Tail)
where
	Tail: HListIntoIterator<Head>,
{
	type Iterator = iter::Chain<iter::Once<Head>, Tail::Iterator>;

	fn into_iter(self) -> Self::Iterator {
		iter::once(self.0).chain(self.1.into_iter())
	}
}

/// "Builder" for storage checks
pub struct StorageChecker<UncheckedStorages, CheckedStorages>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
{
	expected_changes: ExpectedChangesOf<CheckedStorages>,
	_marker: PhantomData<fn() -> UncheckedStorages>,
}

impl<UncheckedStorages: PalletStorageHList, CheckedStorages: PalletStorageHList>
	StorageChecker<UncheckedStorages, CheckedStorages>
{
	/// Adds a check for the storage `T`, moving it from `UncheckedStorages` to `CheckedStorages` in
	/// doing so.
	pub(crate) fn check_storage<T: CheckStorage, Index>(
		self,
		t_value: <T::Value as Diffable>::ChangeSet,
	) -> StorageChecker<<UncheckedStorages as Find<T, Index>>::Remainder, (T, CheckedStorages)>
	where
		<T::Value as Diffable>::ChangeSet: Debug,
		UncheckedStorages: Find<T, Index>,
		<UncheckedStorages as Find<T, Index>>::Remainder: PalletStorageHList,
	{
		StorageChecker {
			expected_changes: self.expected_changes.prepend(t_value),
			_marker: PhantomData,
		}
	}
}

impl<PalletStorages: PalletStorageHList> StorageChecker<PalletStorages, ()> {
	/// Creates a new [`StorageTester`] with the provided pallet storages and with all storages
	/// unchecked.
	#[allow(clippy::new_without_default)]
	pub fn new() -> StorageChecker<PalletStorages, ()> {
		StorageChecker { expected_changes: (), _marker: PhantomData }
	}
}

impl<UncheckedStorages, CheckedStorages> StorageChecker<UncheckedStorages, CheckedStorages>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	UncheckedStorages::NamesOutput: HZippable<AssertionOutputOf<UncheckedStorages>>,
	CheckedStorages::NamesOutput: HZippable<AssertionOutputOf<CheckedStorages>>,
	Zipped<UncheckedStorages::NamesOutput, AssertionOutputOf<UncheckedStorages>>:
		Concat<Zipped<CheckedStorages::NamesOutput, AssertionOutputOf<CheckedStorages>>>,
{
	#[allow(clippy::type_complexity)]
	pub(crate) fn check<F: FnOnce()>(
		self,
		f: F,
	) -> Concatenated<
		Zipped<UncheckedStorages::NamesOutput, AssertionOutputOf<UncheckedStorages>>,
		Zipped<CheckedStorages::NamesOutput, AssertionOutputOf<CheckedStorages>>,
	> {
		let unchecked_value_before_f = UncheckedStorages::current_value();
		let checked_value_before_f = CheckedStorages::current_value();

		f();

		// let expected_unchecked_diff = Diff::NotChanged;

		// let buf = String::new();

		// map of storage name to storage check error.
		// let errors = BTreeMap::new();

		// this should be equal to self.input
		let found_checked_diff =
			CheckedStorages::diff_storage_changes_with_expected_changes(checked_value_before_f);

		let checked_assertions = CheckedStorages::names()
			.zip(found_checked_diff.assert_changes_are_as_expected(self.expected_changes));

		// this should result in no changes, assuming the storages haven't been changed. if there
		// have been unaccounted for changes, then this will result in a "failed" diff
		let unchecked_diff =
			UncheckedStorages::diff_storage_changes_with_expected_changes(unchecked_value_before_f);

		let unchecked_assertions =
			UncheckedStorages::names().zip(unchecked_diff.assert_unchanged());

		unchecked_assertions.concat(checked_assertions)
	}
}

// #[cfg(test)]
// mod test {
// 	use std::collections::BTreeMap;

// 	use sp_std::map;

// 	use crate::{diff::MapValueDiff, do_action, CheckStorage};

// 	struct One;
// 	struct Two;
// 	struct Three;

// 	impl CheckStorage for One {
// 		type Value = u8;

// 		const NAME: &'static str = "A";

// 		fn current_value() -> Self::Value {
// 			1
// 		}
// 	}

// 	impl CheckStorage for Two {
// 		type Value = u16;

// 		const NAME: &'static str = "B";

// 		fn current_value() -> Self::Value {
// 			2
// 		}
// 	}

// 	impl CheckStorage for Three {
// 		type Value = BTreeMap<u32, u32>;

// 		const NAME: &'static str = "C";

// 		fn current_value() -> Self::Value {
// 			[(1, 2)].into_iter().collect()
// 		}
// 	}

// 	type HListT = (One, (Two, (Three, ())));
// 	// type HListT = (One, (Two, ()));

// 	#[test]
// 	fn abc() {
// 		do_action::<HListT, _>(|| {
// 			//
// 			//
// 			// do a bunch of cool stuff here!
// 			//
// 			//
// 		})
// 		.check_storage::<Two, _>(3)
// 		.check_storage::<Three, _>(map! { 1 => MapValueDiff::Added(1) })
// 		.assert_storage_changes();

// 		// .check(|| {});
// 		// .check();
// 		// let _: <HList as Find<u16, _>>::Remainder = 1;
// 		// let _: <HList as Find<_, There<There<There<Here>>>>>::Type = 1_u64;
// 		// let _: <HList as Find<u32, _>>::Index = There::<Here>(PhantomData);
// 	}

// 	// start with hlist of pallet storages:
// 	// struct StorageChecker<CheckedStorages, UncheckedStorages>;
// 	// starts off as StorageChecker<(), PalletStoragesHList>;
// 	//
// 	// on each storage check:
// 	// Find<Storage<T>, _>::Remainder becomes the new storage type, add Storage<T> to the checked
// 	//
// 	// maybe:
// 	// storages StorageChecker::build() wraps the CheckedStorages in Check<Storage<T>>, and the
// 	// UncheckedStorages in AssumeNoChanges<Storage<T>>
// }
