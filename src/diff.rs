use core::{convert::Infallible, fmt::Debug};

use frame_support::{traits::Get, BoundedBTreeMap};
use sp_arithmetic::{fixed_point::FixedU64, FixedU128};
use sp_core::sr25519;
use sp_runtime::Perbill;
use sp_std::collections::btree_map::BTreeMap;

// TODO: docs lol
pub trait Diffable: Debug {
	/// This will usually just be `Self`, but some types may with to use more specialized types to
	/// better describe the diff, such as [`OptionDiff`] for `Option` or [`BTreeMap<K,
	/// MapValueDiff<V>>`] for [`BTreeMap`] (and [`BoundedBTreeMap`]).
	type ChangeSet: PartialEq + Debug;

	/// Diff the old value (`self`) with the `new_value`. See [`Diff`] for more information.
	fn diff(self, new_value: Self) -> Diff<Self::ChangeSet>;
}

/// Represents the diff of two values.
///
/// Note that this is done on a "changed or not changed" basis; [`Diff::ChangedTo`] contains the
/// *new value* if it was changed, rather than the "diff" of the value itself:
///
/// ```rust
/// # use change_set::{Diff, Diffable};
/// let a = 15;
/// let b = 10;
/// assert_eq!(a.diff(b), Diff::ChangedTo(10));
/// ```
///
/// This is different from the typical definition of "diff", where one would expect something
/// like this:
///
/// ```rust,ignore
/// # use change_set::{Diff, Diffable};
/// let a = 15;
/// let b = 10;
/// assert_eq!(a.diff(b), Some(-5));
/// ```
///
/// This type can be thought of as a specialized [`Option`].
#[derive(Debug, PartialEq, Eq)]
pub enum Diff<T> {
	/// The value was not changed.
	NotChanged,
	/// The value was changed. Contained is the new value that it was changed to.
	ChangedTo(T),
}

// deriving Default adds a bound to T, so manually impl it for now
impl<T> Default for Diff<T> {
	fn default() -> Self {
		Self::NotChanged
	}
}

/// Describes the diff between the values of two map-like structures.
///
/// Not intended to be used as a standalone diff - this should be used with it's associated key
/// in the map.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum MapValueDiff<T: Diffable> {
	/// The item under this key was not changed between the original map and the updated map.
	#[default]
	NotChanged,
	/// The item under this key in the original map was not found in the updated map.
	Missing,
	/// The item under this key was not in the original map. Contaianed is the un-diffed value,
	/// as there is nothing to diff it against.
	Added(T),
	/// Th item under this key has changed between the original map and the updated map.
	/// Contained is the diff between the original and updated value.
	Changed(T::ChangeSet),
}

impl<K: Ord + Debug, V: PartialEq + Debug + Diffable, S: Get<u32>> Diffable
	for BoundedBTreeMap<K, V, S>
{
	type ChangeSet = <BTreeMap<K, V> as Diffable>::ChangeSet;

	fn diff(self, updated: Self) -> Diff<Self::ChangeSet> {
		self.into_inner().diff(updated.into_inner())
	}
}

impl<K: Ord + Debug, V: PartialEq + Debug + Diffable> Diffable for BTreeMap<K, V> {
	type ChangeSet = BTreeMap<K, MapValueDiff<V>>;

	fn diff(self, mut updated: Self) -> Diff<Self::ChangeSet> {
		let mut map = self
			.into_iter()
			.map(|(k, v)| match updated.remove(&k) {
				Some(maybe_updated) => match v.diff(maybe_updated) {
					Diff::NotChanged => (k, MapValueDiff::NotChanged),
					Diff::ChangedTo(changed) => (k, MapValueDiff::Changed(changed)),
				},
				None => (k, MapValueDiff::Missing),
			})
			.collect::<Self::ChangeSet>();

		map.extend(updated.into_iter().map(|(k, v)| (k, MapValueDiff::Added(v))));

		let map = map
			.into_iter()
			.filter(|(_k, v)| !matches!(v, MapValueDiff::NotChanged))
			.collect::<Self::ChangeSet>();

		if map.is_empty() {
			Diff::NotChanged
		} else {
			Diff::ChangedTo(map)
		}
	}
}

#[cfg(test)]
mod test_btree_map_diff {
	use std::collections::BTreeMap;

	use super::{Diff, Diffable, MapValueDiff};

	#[test]
	fn test_not_changed() {
		let map = [(1, 10), (2, 20), (3, 30)].into_iter().collect::<BTreeMap<u32, u32>>();

		let updated = [(1, 10), (2, 20), (3, 30)].into_iter().collect::<BTreeMap<u32, u32>>();

		assert_eq!(map.diff(updated), Diff::NotChanged);
	}

	#[test]
	fn test_added() {
		let map = [(1, 10), (2, 20), (3, 30)].into_iter().collect::<BTreeMap<u32, u32>>();

		let updated =
			[(1, 10), (2, 20), (3, 30), (4, 40)].into_iter().collect::<BTreeMap<u32, u32>>();

		assert_eq!(
			map.diff(updated),
			Diff::ChangedTo(
				[(4, MapValueDiff::Added(40))].into_iter().collect::<BTreeMap<u32, _>>()
			)
		);
	}
}

impl<T: Diffable + PartialEq + Eq + Debug> Diffable for Option<T> {
	type ChangeSet = OptionDiff<T>;

	fn diff(self, updated: Self) -> Diff<Self::ChangeSet> {
		match (self, updated) {
			// TODO(benluelo): update the ChangeSet macro to generate something like this instead of
			// using unreachable!()
			(None, None) => Diff::NotChanged,
			(None, Some(v)) => Diff::ChangedTo(OptionDiff::WasNoneNowSome(v)),
			(Some(_), None) => Diff::ChangedTo(OptionDiff::WasSomeNowNone),
			(Some(old), Some(new)) => match old.diff(new) {
				Diff::NotChanged => Diff::NotChanged,
				Diff::ChangedTo(changed) => Diff::ChangedTo(OptionDiff::ValueChanged(changed)),
			},
		}
	}
}

/// Describes the diff between two [`Option`]s.
#[derive(Debug, PartialEq, Eq)]
pub enum OptionDiff<T: Diffable> {
	/// The value was previously `Some(x)`, and is now `Some(y)` where `x != y`.
	ValueChanged(T::ChangeSet),
	/// The value was previously `None` but is now `Some`. Contained is the un-diffed value, as
	/// there is nothing to diff it against.
	WasNoneNowSome(T),
	/// The value was previously `Some`, but is now `None`.
	WasSomeNowNone,
}

impl Diffable for () {
	type ChangeSet = Infallible;

	fn diff(self, _: Self) -> Diff<Self::ChangeSet> {
		Diff::NotChanged
	}
}

macro_rules! impl_diff_primitives {
	($ty: ty) => {
		impl Diffable for $ty {
			type ChangeSet = $ty;

			fn diff(self, new_value: Self) -> Diff<Self::ChangeSet> {
				if self == new_value {
					Diff::NotChanged
				} else {
					Diff::ChangedTo(new_value)
				}
			}
		}
	};
}

// unsigned
impl_diff_primitives!(u8);
impl_diff_primitives!(u16);
impl_diff_primitives!(u32);
impl_diff_primitives!(u64);
impl_diff_primitives!(u128);

// signed
impl_diff_primitives!(i8);
impl_diff_primitives!(i16);
impl_diff_primitives!(i32);
impl_diff_primitives!(i64);
impl_diff_primitives!(i128);

// other types that work with this macro
impl_diff_primitives!(Perbill);
impl_diff_primitives!(FixedU128);
impl_diff_primitives!(FixedU64);
impl_diff_primitives!(sr25519::Public);
