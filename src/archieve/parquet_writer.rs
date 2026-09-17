use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::sync::Arc;

fn write_batches_to_parquet(
    batches: &[RecordBatch],
    schema: Arc<Schema>,
    path: &str,
) -> anyhow::Result<()> {
    let file = File::create(path)?;

    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .set_max_row_group_size(1_000_000)
        .build();

    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
    for batch in batches {
        writer.write(batch)?;
    }
    writer.close()?; // flushes the footer - skip this and the file read back empty/corrupt
    Ok(())
}
