mod parquet_writer;
mod parquet_reader;
mod listing;
mod registry;

use arrow::array::{Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn schema() -> Schema {
    Schema::new(vec![
        Field::new("order_id", DataType::Int64, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("amount", DataType::Float64, false),
    ])
}

fn build_batch(rows: &[(i64, String, Option<f64>)]) -> arrow::error::Result<RecordBatch> {
    let mut id_builder = Int64Builder::new();
    let mut status_builder = StringBuilder::new();
    let mut amount_builder = Float64Builder::new();
    for (id, status, amount) in rows {
        id_builder.append_value(*id);
        status_builder.append_value(status);
        match amount {
            Some(v) => amount_builder.append_value(*v),
            None => amount_builder.append_null(),
        }
    }

    RecordBatch::try_new(
        Arc::new(schema()),
        vec![
            Arc::new(id_builder.finish()),
            Arc::new(status_builder.finish()),
            Arc::new(amount_builder.finish()),
        ],
    )
}
