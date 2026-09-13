use datafusion::prelude::*;

#[tokio::main]
async fn main () -> datafusion::error::Result<()> {
    let ctx = SessionContext::new();

    ctx.register_parquet("orders_archieve","archieve/orders.parquet", ParquetReadOptions::default()).await?;

    let df = ctx.sql(
        "SELECT status, count(*) FROM orders_archieve GROUP BY status"
        ).await?;
    
    df.show().await?;

    let explain = ctx.sql("EXPLAIN ANALYZE SELECT * FROM orders_archieve WHERE status = 'shipped'").await?;
    explain.show().await?;

    Ok(())
}
