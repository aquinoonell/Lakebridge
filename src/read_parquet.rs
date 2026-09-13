use std::fs::File;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use arrow::record_batch::RecordBatch;

fn read_parquet_to_batches(path: &str) -> anyhow::Result<Vec<RecordBatch>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;

    // builder.metadata().row_group(0).column(0).statistics() lets you inspect
    // min/max/null_count directly, if you want to see pruning inputs firshand.

    let reader = builder.build()?;

    let mut batches = Vec::new();
    for batch in reader  {
        batches.push(batch?);
    }

    Ok(batches)
}
