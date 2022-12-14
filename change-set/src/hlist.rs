use core::{fmt::Debug, iter, marker::PhantomData};

use crate::{CheckStorage, Diff, Diffable};

pub(crate) type Zipped<A, B> = <A as HZippable<B>>::Zipped;
pub(crate) type Concatenated<A, B> = <A as Concat<B>>::Output;
pub(crate) type AssertionOutputOf<T> =
	<<T as PalletStorageHList>::Diff as AssertDiffHList>::AssertionOutput;
pub(crate) type ExpectedChangesOf<T> =
	<<T as PalletStorageHList>::Diff as AssertDiffHList>::ExpectedChanges;

/// Generic heterogenous list trait.
pub trait HList: Sized {
	fn prepend<H>(self, h: H) -> (H, Self) {
		(h, self)
	}
}

impl HList for () {}

impl<Head, Tail> HList for (Head, Tail) where Tail: HList {}

/// Used as an index into an [`HList`].
///
/// `Here` is 0, pointing to the head of the [`HList`].
///
/// Users should normally allow type inference to create this type
pub enum Here {}

/// Used as an index into an [`HList`].
///
/// `There<T>` is 1 + `T`.
///
/// Users should normally allow type inference to create this type.
pub struct There<T>(PhantomData<T>);

/// Find an element in an [`HList`] by it's type. `I` can be inferred as long as the elements in the
/// [`HList`] are unique.
pub trait Find<T, I> {
	type Remainder;
}

impl<T, Tail> Find<T, Here> for (T, Tail) {
	type Remainder = Tail;
}

impl<Head, T, Tail, TailIndex> Find<T, There<TailIndex>> for (Head, Tail)
where
	Tail: Find<T, TailIndex>,
{
	type Remainder = (Head, <Tail as Find<T, TailIndex>>::Remainder);
}

// Zip two [`HList`]s together. Only works with [`HList`]s of the same length.
pub trait HZippable<Other> {
	type Zipped: HList;

	/// Zip this [`HList`] with another one.
	fn zip(self, other: Other) -> Self::Zipped;
}

impl HZippable<()> for () {
	type Zipped = ();

	fn zip(self, _other: ()) -> Self::Zipped {}
}

impl<Head1, Tail1, Head2, Tail2> HZippable<(Head2, Tail2)> for (Head1, Tail1)
where
	Tail1: HZippable<Tail2>,
{
	type Zipped = ((Head1, Head2), Tail1::Zipped);

	fn zip(self, other: (Head2, Tail2)) -> Self::Zipped {
		((self.0, other.0), self.1.zip(other.1))
	}
}

/// Diff [`HList`] trait
///
/// Will be implemented on an [`HList`] of `(Diff<T>, (Diff<U>, (..., ())))`
pub trait AssertDiffHList: HList {
	type ExpectedChanges: HList;
	type AssertionOutput: HList;

	fn assert_unchanged(self) -> Self::AssertionOutput;

	fn assert_changes_are_as_expected(
		self,
		expected: Self::ExpectedChanges,
	) -> Self::AssertionOutput;
}

impl AssertDiffHList for () {
	type AssertionOutput = ();
	type ExpectedChanges = ();

	fn assert_unchanged(self) -> Self::AssertionOutput {}

	fn assert_changes_are_as_expected(
		self,
		_expected: Self::ExpectedChanges,
	) -> Self::AssertionOutput {
	}
}

impl<Head: PartialEq + Debug, Tail> AssertDiffHList for (Diff<Head>, Tail)
where
	Tail: AssertDiffHList,
{
	// Option<String> should be replaced with a better type at some point, but this works for now
	type AssertionOutput = (Option<String>, Tail::AssertionOutput);
	type ExpectedChanges = (Head, Tail::ExpectedChanges);

	fn assert_unchanged(self) -> Self::AssertionOutput {
		let output = match self.0 {
			Diff::NotChanged => None,
			Diff::ChangedTo(change) => Some(format!("Expected no changes, found {change:#?}")),
		};

		(output, self.1.assert_unchanged())
	}

	fn assert_changes_are_as_expected(
		self,
		expected: Self::ExpectedChanges,
	) -> Self::AssertionOutput {
		let output = match self.0 {
			Diff::NotChanged => {
				Some(format!("expected change of {:#?}, found no changes", expected.0))
			},
			Diff::ChangedTo(change) => {
				if change == expected.0 {
					None
				} else {
					Some(format!("expected change of {:#?}, found {change:#?}", expected.0))
				}
			},
		};

		(output, self.1.assert_unchanged())
	}
}

/// Concat two [`HList`]s
pub trait Concat<Rhs> {
	type Output;

	fn concat(self, rhs: Rhs) -> Self::Output;
}

impl<Rhs> Concat<Rhs> for ()
where
	Rhs: HList,
{
	type Output = Rhs;

	fn concat(self, rhs: Rhs) -> Rhs {
		rhs
	}
}

impl<Head, Tail, Rhs> Concat<Rhs> for (Head, Tail)
where
	Tail: Concat<Rhs>,
	Rhs: HList,
{
	type Output = (Head, <Tail as Concat<Rhs>>::Output);

	fn concat(self, rhs: Rhs) -> Self::Output {
		(self.0, self.1.concat(rhs))
	}
}

/// The type of the [`HList`]'s [`Diffable::ChangeSet`].
pub trait DiffableHList {
	type ChangeSet;
}

impl DiffableHList for () {
	type ChangeSet = ();
}

impl<Head: Diffable, Tail> DiffableHList for (Head, Tail)
where
	Tail: DiffableHList,
{
	type ChangeSet = (<Head as Diffable>::ChangeSet, <Tail as DiffableHList>::ChangeSet);
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

/// [`HList`] trait specific to the pallet storages. This is used to define all the storages that
/// will be checked by the [`AssertableDiffableStorageAction`].
pub trait PalletStorageHList: HList {
	type NamesOutput;
	type CurrentValue: DiffableHList<ChangeSet = ExpectedChangesOf<Self>>;
	type Diff: AssertDiffHList;

	/// The names of the storages in this [`HList`].
	fn names() -> Self::NamesOutput;

	fn current_value() -> Self::CurrentValue;

	fn diff_storage_changes_with_expected_changes(expected: Self::CurrentValue) -> Self::Diff;
}

impl PalletStorageHList for () {
	type CurrentValue = ();
	type Diff = ();
	type NamesOutput = ();

	fn names() -> Self::NamesOutput {}

	fn current_value() -> Self::CurrentValue {}

	fn diff_storage_changes_with_expected_changes(_: Self::CurrentValue) -> Self::Diff {}
}

impl<Head, Tail> PalletStorageHList for (Head, Tail)
where
	Head: CheckStorage,
	<Head::Value as Diffable>::ChangeSet: Debug,
	Tail: PalletStorageHList,
{
	type CurrentValue = (Head::Value, Tail::CurrentValue);
	type Diff = (Diff<<Head::Value as Diffable>::ChangeSet>, Tail::Diff);
	type NamesOutput = ((&'static str, &'static str), Tail::NamesOutput);

	fn names() -> Self::NamesOutput {
		(Head::name(), Tail::names())
	}

	fn current_value() -> Self::CurrentValue {
		(Head::current_value(), Tail::current_value())
	}

	fn diff_storage_changes_with_expected_changes(expected: Self::CurrentValue) -> Self::Diff {
		(
			Head::diff_storage_changes_with_expected_changes(expected.0),
			Tail::diff_storage_changes_with_expected_changes(expected.1),
		)
	}
}
