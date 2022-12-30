use core::app::App;

use anyhow::Result;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new().init().unwrap();
    let mut app = App::new();
    app.run().await;

    Ok(())
}
