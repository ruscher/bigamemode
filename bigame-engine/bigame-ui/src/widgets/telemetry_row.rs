//! Telemetry display row: CPU/GPU gauge with live sparkline.

use libadwaita as adw;
use adw::prelude::*;

use crate::widgets::sparkline::{self, SparkHandle};

/// Build a telemetry display row with embedded sparkline.
///
/// Returns `(ActionRow, SparkHandle)`. Push numeric values into the handle
/// each poll tick to animate the sparkline.
#[must_use]
pub fn build(title: &str, initial_value: &str) -> (adw::ActionRow, SparkHandle) {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(initial_value)
        .build();
    row.add_css_class("telemetry-row");

    let spark = sparkline::build();
    row.add_suffix(&spark.area);

    (row, spark)
}
