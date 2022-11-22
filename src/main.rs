mod cli;
mod config;

use crate::cli::GhDashCli;
use crate::config::GhConfig;
use clap::Parser;
use ghdash::Dashboard;

type Error = ghdash::Error;
const APP_NAME: &str = clap::crate_name!();

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = GhDashCli::parse();

    let config_name = args.config();
    let config_name = config_name.as_deref();

    let mut cfg: GhConfig = confy::load(APP_NAME, config_name)?;

    if let Some(user) = args.user() {
        cfg.set_user(user.as_str());
        confy::store(APP_NAME, config_name, cfg.clone())?;
    }

    if let Some(token) = args.token() {
        cfg.set_token(token.as_str());
        confy::store(APP_NAME, config_name, cfg.clone())?;
    }

    dbg!(&cfg);

    let dashboard = Dashboard::new(cfg.user().as_str(), cfg.token().as_str()).await?;

    dbg!(dashboard);

    Ok(())
}
