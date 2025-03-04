use super::*;
use sp_std::marker::PhantomData;

use frame_support::{
	log, migration::storage_key_iter, pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight, StoragePrefixedMap,
};
use pallet_asset_registry::{AssetLocations, LocationAssets};
use polkadot_xcm::v3::MultiLocation;

pub struct OnRuntimeUpgradeMigration;
impl OnRuntimeUpgrade for OnRuntimeUpgradeMigration {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!("PreMigrate Transaction Pause Pallet start");
		let tx_pause_state = pallet_transaction_pause::migration::v1::Migration::<Runtime>::pre_upgrade()?;
		log::info!("PreMigrate Transaction Pause Pallet end");

		log::info!("PreMigrate Collator Rewards Pallet start");
		pallet_collator_rewards::migration::v1::pre_migrate::<Runtime>();
		log::info!("PreMigrate Collator Rewards Pallet end");

		log::info!("PreMigrate Genesis History Pallet start");
		pallet_genesis_history::migration::v1::pre_migrate::<Runtime>();
		log::info!("PreMigrate Genesis History Pallet end");

		Ok(tx_pause_state)
	}

	fn on_runtime_upgrade() -> Weight {
		let mut weight: Weight = Weight::zero();

		log::info!("Migrate Transaction Pause Pallet to v1 start");
		weight =
			weight.saturating_add(pallet_transaction_pause::migration::v1::Migration::<Runtime>::on_runtime_upgrade());
		log::info!("Migrate Transaction Pause Pallet to v1 end");

		log::info!("Migrate Collator Rewards Pallet to v1 start");
		weight = weight.saturating_add(pallet_collator_rewards::migration::v1::migrate::<Runtime>());
		log::info!("Migrate Collator Rewards Pallet to v1 end");

		log::info!("Migrate Genesis History Pallet to v1 start");
		weight = weight.saturating_add(pallet_genesis_history::migration::v1::migrate::<Runtime>());
		log::info!("Migrate Genesis History Pallet to v1 end");

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		log::info!("PostMigrate Transaction Pause Pallet start");
		pallet_transaction_pause::migration::v1::Migration::<Runtime>::post_upgrade(state)?;
		log::info!("PostMigrate Transaction Pause Pallet end");

		log::info!("PostMigrate Collator Rewards Pallet start");
		pallet_collator_rewards::migration::v1::post_migrate::<Runtime>();
		log::info!("PostMigrate Collator Rewards Pallet end");

		log::info!("PostMigrate Genesis History Pallet start");
		pallet_genesis_history::migration::v1::post_migrate::<Runtime>();
		log::info!("PostMigrate Genesis History Pallet end");

		Ok(())
	}
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
pub struct AssetLocationV2(pub polkadot_xcm::v2::MultiLocation);

pub struct MigrateRegistryLocationToV3<T>(PhantomData<T>);
impl<T: pallet_asset_registry::Config> OnRuntimeUpgrade for MigrateRegistryLocationToV3<T>
where
	AssetLocation: Into<T::AssetNativeLocation>,
{
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "asset-registry",
			"MigrateRegistryLocationToV3::on_runtime_upgrade: migrating asset locations to v3"
		);

		let mut weight: Weight = Weight::zero();

		AssetLocations::<T>::translate(|_asset_id, old_location: AssetLocationV2| {
			weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
			let new_multi_loc: MultiLocation = old_location.0.try_into().expect("xcm::v1::MultiLocation");
			let new_location: T::AssetNativeLocation = AssetLocation(new_multi_loc).into();
			Some(new_location)
		});

		let module_prefix = LocationAssets::<T>::module_prefix();
		let storage_prefix = LocationAssets::<T>::storage_prefix();
		let old_data = storage_key_iter::<AssetLocationV2, T::AssetId, Blake2_128Concat>(module_prefix, storage_prefix)
			.drain()
			.collect::<Vec<_>>();
		for (old_location, asset_id) in old_data {
			weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
			let new_multi_loc: MultiLocation = old_location.0.try_into().expect("xcm::v1::MultiLocation");
			let new_location: T::AssetNativeLocation = AssetLocation(new_multi_loc).into();
			LocationAssets::<T>::insert(new_location, asset_id);
		}
		weight
	}
}

pub struct XcmRateLimitMigration;
impl OnRuntimeUpgrade for XcmRateLimitMigration {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		frame_support::log::info!("PreMigrate Asset Registry Pallet start");
		pallet_asset_registry::migration::v1::pre_migrate::<Runtime>();
		frame_support::log::info!("PreMigrate Asset Registry Pallet end");

		Ok(vec![])
	}

	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "runtime::asset-registry",
			"XcmRateLimitMigration::on_runtime_upgrade: migrating asset details to include xcm rate limit"
		);

		pallet_asset_registry::migration::v1::migrate::<Runtime>()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		frame_support::log::info!("PostMigrate Asset Registry Pallet start");
		pallet_asset_registry::migration::v1::post_migrate::<Runtime>();
		frame_support::log::info!("PostMigrate Asset Registry Pallet end");
		Ok(())
	}
}
