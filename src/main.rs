use color_eyre::Result;

pub mod app;
pub mod config;
pub mod openstack;
pub mod state;

use crate::{
    app::App,
};

async fn tokio_main() -> Result<()> {
    let mut app = App::new();
    let terminal = ratatui::init();
    let app_result = app.run(terminal).await;
    ratatui::restore();

    app_result
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}

