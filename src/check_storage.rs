use core::fmt::Debug;

use frame_support::{
	pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue},
	storage::types::QueryKindTrait,
	traits::{Get, StorageInstance},
	ReversibleStorageHasher, StorageHasher,
};
use parity_scale_codec::FullCodec;
use sp_std::collections::btree_map::BTreeMap;

use crate::diff::{Diff, Diffable};

pub trait CheckStorage {
	type Value: Diffable;

	// const NAME: &'static str;

	fn name() -> (&'static str, &'static str);

	fn current_value() -> Self::Value;

	fn diff_storage_changes_with_expected_changes(
		expected: Self::Value,
	) -> Diff<<Self::Value as Diffable>::ChangeSet> {
		expected.diff(Self::current_value())
	}
}

impl<Prefix, Value, QueryKind, OnEmpty> CheckStorage
	for StorageValue<Prefix, Value, QueryKind, OnEmpty>
where
	Prefix: StorageInstance,
	Value: FullCodec + Diffable,
	QueryKind: QueryKindTrait<Value, OnEmpty>,
	QueryKind::Query: Diffable,
	OnEmpty: Get<QueryKind::Query> + 'static,
{
	type Value = QueryKind::Query;

	// const NAME: &'static str = Prefix::STORAGE_PREFIX;

	fn name() -> (&'static str, &'static str) {
		(Prefix::pallet_prefix(), Prefix::STORAGE_PREFIX)
	}

	fn current_value() -> Self::Value {
		Self::get()
	}
}

impl<Prefix, Hasher, Key, Value, QueryKind, OnEmpty, MaxValues> CheckStorage
	for StorageMap<Prefix, Hasher, Key, Value, QueryKind, OnEmpty, MaxValues>
where
	Prefix: StorageInstance,
	Hasher: StorageHasher + ReversibleStorageHasher,
	Key: FullCodec + Debug + Ord,
	Value: FullCodec + PartialEq + Diffable,
	QueryKind: QueryKindTrait<Value, OnEmpty>,
	QueryKind::Query: Diffable,
	OnEmpty: Get<QueryKind::Query> + 'static,
	MaxValues: Get<Option<u32>>,
{
	type Value = BTreeMap<Key, Value>;

	fn name() -> (&'static str, &'static str) {
		(Prefix::pallet_prefix(), Prefix::STORAGE_PREFIX)
	}

	fn current_value() -> Self::Value {
		Self::iter().collect::<BTreeMap<_, _>>()
	}
}

impl<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty, MaxValues> CheckStorage
	for StorageDoubleMap<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty, MaxValues>
where
	Prefix: StorageInstance,
	Hasher1: StorageHasher + ReversibleStorageHasher,
	Key1: FullCodec + Debug + Ord,
	Hasher2: StorageHasher + ReversibleStorageHasher,
	Key2: FullCodec + Debug + Ord,
	Value: FullCodec + PartialEq + Diffable,
	QueryKind: QueryKindTrait<Value, OnEmpty>,
	OnEmpty: Get<QueryKind::Query> + 'static,
	MaxValues: Get<Option<u32>>,
{
	// REVIEW(benluelo): Would `BTreeMap<(Key1, Key2)>, Value>` have better UX?
	type Value = BTreeMap<Key1, BTreeMap<Key2, Value>>;

	fn name() -> (&'static str, &'static str) {
		(Prefix::pallet_prefix(), Prefix::STORAGE_PREFIX)
	}

	fn current_value() -> Self::Value {
		let mut found_map = BTreeMap::new();

		for (k1, k2, v) in Self::iter() {
			dbg!(&k1, &k2);
			found_map.entry(k1).or_insert_with(BTreeMap::<Key2, Value>::new).insert(k2, v);
		}

		found_map
	}
}
