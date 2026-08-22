//! Asset database and import discovery.

mod database;

pub use database::{AssetEntry, AssetKind, AssetDatabase};

use bevy::prelude::*;

pub fn install_asset_database(app: &mut App) {
    app.init_resource::<AssetDatabase>()
        .add_systems(Startup, database::initial_scan)
        .add_systems(Update, database::refresh_on_request);
}
