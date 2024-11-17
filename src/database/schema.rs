use super::initialization::VECTOR_SIZE;
use arrow_schema::{DataType, Field, Schema};
use std::sync::Arc;

lazy_static::lazy_static! {
    pub(crate) static ref PATTERNS_SCHEMA: Schema = Schema::new(vec![
        Field::new("endpoint_id", DataType::Utf8, false),
        Field::new("pattern", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                VECTOR_SIZE,
            ),
            false,
        ),
    ]);
}
