//! Integrations test for --counter component with blobstore-filesystem plugin
//!
//! This test demonstrates component-to-component linking by:
//! 1. Running the blobstore-filesystem plugin as a component that exports wasi:blobstore
//! 2. Running the http-counter component that imports wasi:blobstore
//! 3. Verifying that the http-counter can use the blobstore-filesystem implementation
//! 4. Testing the component resolution system that links them together

use std::path::PathBuf;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context as _, Result, ensure};
use tokio::time::timeout;
use tracing::{debug, info};

use wash_runtime::engine::Engine;
use wash_runtime::host::http::{DevRouter, HttpServer};
use wash_runtime::host::{HostApi, HostBuilder};
use wash_runtime::types::{
    Component, HostPathVolume, LocalResources, Volume, VolumeMount, VolumeType, Workload,
    WorkloadStartRequest,
};
use wash_runtime::wit::WitInterface;

use wash::plugin::PluginManager;

/// Ensure the http-fs-hello component runs properly
#[tokio::test]
async fn test_int_http_fs_hello_component() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Read the component bytes from disk (built during build.rs)
    let component_bytes = tokio::fs::read(PathBuf::from(env!("COMPONENT_HTTP_FS_HELLO_PATH")))
        .await
        .context("failed to read component bytes")?;

    // TODO: we really should be using 0 here for the port, and extracting the resolved
    // random port *after* startup of the HTTP plugin, from the HTTP plugin.
    let port = find_available_port()
        .await
        .context("failed to find random available port")?;
    let addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .context("failed to build socket addr")?;
    let http_plugin = HttpServer::new(DevRouter::default(), addr);

    let plugin_manager = PluginManager::default();

    // Build WebAssembly engine
    let engine = Engine::builder()
        .build()
        .context("failed to build engine")?;

    // Build wasmCloud custom host
    let host = HostBuilder::new()
        .with_engine(engine.clone())
        .with_http_handler(Arc::new(http_plugin))
        .with_plugin(Arc::new(plugin_manager))?
        .build()
        .context("failed to build host")?;

    // Start the wasmCloud host
    let host = host.start().await.context("failed to start host")?;
    debug!("host started, HTTP server listening on {addr}");

    // Create a temporary directory for http-fs-hello to use
    let http_fs_dir = tempfile::tempdir().context("Failed to create temp dir for blobstore")?;
    debug!(
        "http-fs-hello directory at: {}",
        http_fs_dir.path().display()
    );

    let randomized_text_content = format!(
        "text content, with a random UUID: {}",
        uuid::Uuid::now_v7().to_string()
    );
    tokio::fs::write(
        http_fs_dir.path().join("sample.txt"),
        &randomized_text_content,
    )
    .await
    .context("failed to write sample text file")?;

    // Create a workload with BOTH components:
    let req = WorkloadStartRequest {
        workload_id: uuid::Uuid::now_v7().to_string(),
        workload: Workload {
            namespace: "test".to_string(),
            name: "http-workload".to_string(),
            annotations: HashMap::new(),
            service: None,
            components: vec![Component {
                bytes: bytes::Bytes::from(component_bytes),
                local_resources: LocalResources {
                    memory_limit_mb: 256,
                    cpu_limit: 2,
                    volume_mounts: vec![VolumeMount {
                        name: "data".into(),
                        mount_path: "/data".into(),
                        read_only: true,
                    }],
                    ..Default::default()
                },
                ..Default::default() // pool_size: 2,
                                     // max_invocations: 100,
            }],
            // Host interfaces that the workload needs
            host_interfaces: vec![
                WitInterface {
                    namespace: "wasi".to_string(),
                    package: "http".to_string(),
                    interfaces: ["incoming-handler".to_string()].into_iter().collect(),
                    version: None,
                    config: {
                        let mut config = HashMap::new();
                        config.insert("host".to_string(), "test".to_string());
                        config
                    },
                },
                WitInterface {
                    namespace: "wasmcloud".to_string(),
                    package: "wash".to_string(),
                    interfaces: ["types".to_string()].into_iter().collect(),
                    version: Some(semver::Version::parse("0.0.2").unwrap()),
                    config: HashMap::new(),
                },
            ],
            volumes: vec![Volume {
                name: "data".to_string(),
                volume_type: VolumeType::HostPath(HostPathVolume {
                    local_path: format!("{}", http_fs_dir.path().display()),
                }),
            }],
        },
    };

    // Star the workload
    host.workload_start(req)
        .await
        .context("Failed to start workload with component linking")?;

    let client = reqwest::Client::new();

    // Test the root (/) endoint endpoint
    let home_resp = timeout(
        Duration::from_secs(10),
        client.get(format!("http://{addr}/")).send(),
    )
    .await
    .context("request timed out")?
    .context("request failed")?;

    ensure!(
        home_resp.status().is_success(),
        "response should have succeeded"
    );

    let home_resp_body = home_resp
        .text()
        .await
        .context("failed to get response body")?;
    info!(home_resp_body, "received HTTP response from /");
    assert_eq!(home_resp_body, "Hello!\n");

    // Test the read-file (/read-file) endpoint
    let read_file_resp = timeout(
        Duration::from_secs(10),
        client.get(format!("http://{addr}/read-file")).send(),
    )
    .await
    .context("request timed out")?
    .context("request failed")?;

    ensure!(
        read_file_resp.status().is_success(),
        "response should have succeeded"
    );

    let read_file_resp_body = read_file_resp
        .text()
        .await
        .context("failed to get response body")?;
    info!(read_file_resp_body, "received HTTP response from /read-file");
    assert_eq!(read_file_resp_body, randomized_text_content);

    Ok(())
}

/// Find an available port by binding to a random port (0) and returning the assigned port.
///
/// NOTE: this function is vulnerable to races as the random port could be taken in between
/// assignment and use.
async fn find_available_port() -> Result<u16> {
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    Ok(addr.port())
}
