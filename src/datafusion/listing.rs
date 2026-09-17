use datafusion::datasource::file_format::parquet::ParquetFormat;
use datafusion::datasource::listing::{ListingOptions, ListingTable, ListingTableConfig, ListingTableUrl};
use datafusion::prelude::SessionContext;
use std::sync::Arc;

pub async fn register_archive(ctx: &SessionContext) -> datafusion::error::Result<()> {
    // Fixed: "archieve" → "archive"
    let table_path = ListingTableUrl::parse("archive/")?;

    let listing_options = ListingOptions::new(Arc::new(ParquetFormat::default()))
        .with_file_extension(".parquet");
        // TODO: add .with_table_partition_cols(vec![("year", DataType::Int32), ("month", DataType::Int32)])
        // to enable partition pruning on Hive-style directories

    let config = ListingTableConfig::new(table_path)
        .with_listing_options(listing_options)
        .infer_schema(&ctx.state())
        .await?;

    // Fixed: "orders_archieve" → "orders_archive"
    ctx.register_table("orders_archive", Arc::new(ListingTable::try_new(config)?))?;
    Ok(())
}
