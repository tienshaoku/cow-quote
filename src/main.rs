use cow_vs_uni as cvu;
use cvu::run;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    run().await
}
