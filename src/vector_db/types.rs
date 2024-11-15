use anyhow::{Context, Result as AnyhowResult};
use arrow::record_batch::RecordBatch;
use arrow_array::{Array, Float32Array, StringArray};

pub struct BatchColumns<'a> {
    pub endpoint_id_column: &'a StringArray,
    pub pattern_column: &'a StringArray,
    pub distance_column: &'a Float32Array,
}

pub fn extract_columns(rb: &RecordBatch) -> AnyhowResult<BatchColumns<'_>> {
    Ok(BatchColumns {
        endpoint_id_column: rb
            .column_by_name("endpoint_id")
            .context("endpoint_id column not found")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Failed to downcast endpoint_id column")?,
        pattern_column: rb
            .column_by_name("pattern")
            .context("pattern column not found")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Failed to downcast pattern column")?,
        distance_column: rb
            .column_by_name("_distance")
            .context("_distance column not found")?
            .as_any()
            .downcast_ref::<Float32Array>()
            .context("Failed to downcast distance column")?,
    })
}
