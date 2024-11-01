use std::sync::Arc;
use arrow_array::{Int32Array, FixedSizeListArray};
use arrow_array::types::Float32Type;
use arrow_array::RecordBatch;
use arrow_schema::{DataType, Field, Schema};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, DistanceType};
use arrow::record_batch::RecordBatchIterator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a local LanceDB instance
    let db = connect("data/mydb").execute().await?;

    // Define the schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                128
            ),
            true,
        ),
    ]));

    // Create sample data
    let id_array = Arc::new(Int32Array::from_iter_values(0..256));
    let vector_array = Arc::new(FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        (0..256).map(|_| Some(vec![Some(1.0); 128])),
        128,
    ));

    // Create a RecordBatch
    let record_batch = RecordBatch::try_new(
        schema.clone(),
        vec![id_array, vector_array],
    )?;

    // Create the table using RecordBatchIterator
    db.create_table(
        "my_table",
        Box::new(RecordBatchIterator::new(
            vec![Ok(record_batch)],
            schema
        )),
    )
    .execute()
    .await?;

    println!("Table created successfully!");

    Ok(())
}
