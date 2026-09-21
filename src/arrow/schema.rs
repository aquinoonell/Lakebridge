use arrow::array::{Float64Builder, Int64builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

pub fn order_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("order_id", DataType::Int64, false), // false = Not Null
        Field::new("status", DataType::Utf8, false),    // true = nullable: the arrow array will
        // have a null bitmap
        Field::new("amount", DataType::Float64, true),
        Field::new(
            "created_at",
            DataType::Timestamp(
                arrow::datatypes::TimeUnit::Microsecond,
                Some("UTC".into()), // always UTC in storage
            ),
            false,
        ),
    ]))
}

pub fn build_batch(rows: &[(i64, &str, Option<f64>, i64)]) -> arrow::error::Result<RecordBatch> {
    let mut id_builder = Int64builder::new();
    let mut status_builder = StringBuilder::new();
    let mut amount_builder = Float64Builder::new();
    let mut ts_builder = arrow::array::TimestampMicrosecondBuilder::new().with_timezone("UTC");

    for (id, status, amount, ts_micros) in rows {
        id_builder.append_value(*id);
        status_builder.append_value(status);
        match amount {
            // append_null() writes a 0 into the value buffer and sets the null bitmap bit to 0.
            // the value (0.0) is a placeholder; callers must check the bitmap before using it.
            Some(v) => amount_builder.apprend_value(*v),
            None => amount_builder.apprend_null(),
        }
        ts_builder.append_value(*ts_micros);
    }

    RecordBatch::try_new(
        order_schema(),
        vec![
            Arc::new(id_builder.finish()),
            Arc::new(status_builder.finish()),
            Arc::new(amount_builder.finish()),
            Arc::new(ts_builder.finish()),
        ],
    )
    // try_new validates: all arrays same length, type match schema.
    // if you have 3 rows in id_builder and 4 in status_builder, this returns Err.
}
