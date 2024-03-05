mod cli;
mod config;

use crate::cli::{Commands, GhDashCli};
use crate::config::GhConfig;
use clap::Parser;
use ghdash::{get_logging, Dashboard, DockerConnection, Error};

const APP_NAME: &str = clap::crate_name!();

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = GhDashCli::parse();

    get_logging(args.logging.log_level_filter()).await?;

    let config_name = args.config;
    let config_name = config_name.as_deref();

    //1. Load the configuration file for default (known) settings
    let mut cfg: GhConfig = confy::load(APP_NAME, config_name)?;

    //2. Check environment for values of user and token
    cfg.try_env_user(APP_NAME);
    cfg.try_env_token(APP_NAME);

    //3. Check command line for values of user and token (save if found)
    if let Some(user) = args.user {
        cfg.set_user(user.as_str());
        confy::store(APP_NAME, config_name, cfg.clone())?;
    }

    if let Some(token) = args.token {
        cfg.set_token(token.as_str());
        confy::store(APP_NAME, config_name, cfg.clone())?;
    }

    match args.command {
        Commands::List => {
            let docker_connection = ghdash::connect_docker().await;
            match docker_connection {
                DockerConnection::Connection(docker) => {
                    println!("Got a connection: {:#?}", docker);
                    let containers = docker
                        .list_containers(Some(
                            bollard::container::ListContainersOptions::<String> {
                                all: true,
                                // filters,
                                ..Default::default()
                            },
                        ))
                        .await
                        .unwrap();

                    println!("List of Containers and Status");

                    for container in containers {
                        println!(
                            "-> Name {:?}\tStatus {:?}",
                            container.names, container.status
                        );
                    }
                    println!();
                }
                DockerConnection::NoConnection => println!("No docker connection"),
            }
        }
    }

    let dashboard = Dashboard::builder(cfg.user().as_str(), cfg.token().as_str())?
        .set_repo_scope(args.repositories.unwrap_or_default())
        .finish()
        .await?;

    print!("{dashboard}");

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
