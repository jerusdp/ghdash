use std::collections::HashMap;

use futures_util::StreamExt;

use bollard::{
    container::{
        CreateContainerOptions, StartContainerOptions, StopContainerOptions, WaitContainerOptions,
    },
    models::PortBinding,
    secret::HostConfig,
    Docker,
};

const DOCKER_SOCKET: &str = "/home/gorta/.docker/desktop/docker.sock";
const TRACER_NAME: &str = "zipkin_tester";
const TRACER_IMAGE: &str = "openzipkin/zipkin";

#[tokio::test]
async fn with_zipkin_tests() {
    let docker =
        bollard::Docker::connect_with_unix(DOCKER_SOCKET, 120, bollard::API_DEFAULT_VERSION)
            .unwrap();

    ensure_container_started(&docker, TRACER_NAME, TRACER_IMAGE).await;

    trycmd::TestCases::new()
        .case("tests/logging/*.trycmd")
        .insert_var("[MESSAGE]", "tracing and logging")
        .unwrap();

    stop_container(&docker, TRACER_NAME).await;
}

async fn ensure_container_started(docker: &Docker, name: &str, image: &str) {
    if !start_container(docker, name).await {
        create_container(docker, name, image).await;
        start_container(docker, name).await;
    }
}

async fn start_container(docker: &Docker, name: &str) -> bool {
    match docker
        .start_container(
            name,
            Some(StartContainerOptions::<String> {
                ..Default::default()
            }),
        )
        .await
    {
        Ok(_) => true,
        Err(e) => match e {
            bollard::errors::Error::DockerResponseServerError {
                status_code,
                message: _,
            } => status_code == 304,
            _ => return false,
        },
    };

    while reqwest::get("http://localhost:9411/api/v2/services")
        .await
        .is_err()
    {}
    true
}

async fn create_container(docker: &Docker, container: &str, image: &str) {
    let options = Some(CreateContainerOptions {
        name: container,
        platform: None,
    });

    let port = "9411";

    let mut mapping = HashMap::new();
    mapping.insert(
        String::from("9411/tcp"),
        Some(vec![PortBinding {
            host_ip: Some("".to_string()),
            host_port: Some(port.to_string()),
        }]),
    );

    let host_config = Some(HostConfig {
        port_bindings: Some(mapping),
        ..Default::default()
    });

    let config = bollard::container::Config {
        image: Some(String::from(image)),
        // exposed_ports,
        host_config,
        ..Default::default()
    };

    dbg!(docker
        .create_container(options, config)
        .await
        .expect("Trying to create the container"));
}

async fn stop_container(docker: &Docker, name: &str) {
    let options = Some(StopContainerOptions { t: 30 });

    docker
        .stop_container(name, options)
        .await
        .expect("Stopping the container");

    let wait_options = Some(WaitContainerOptions {
        condition: "not-running",
    });

    let stream = &mut docker.wait_container(name, wait_options).take(1);

    while let Some(_response) = stream.next().await {}
}
