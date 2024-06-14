use std::{env, net::Ipv4Addr, num::NonZeroU16};

use natpmp_rs::{map_port, protocol::MappingProtocol};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// starts all the tasks, such as the web server, the key refresh, ...
/// ensures all tasks are gracefully shutdown in case of error, ctrl+c or sigterm
async fn start_tasks() -> Result<(), color_eyre::Report> {
    let result = map_port(
        MappingProtocol::TCP,
        unsafe { NonZeroU16::new_unchecked(9_999) },
        // Some(unsafe { NonZeroU16::new_unchecked(10_000) }),
        None,
        None,
        // None,
        Ipv4Addr::new(192, 168, 12, 55).into(),
        None,
    )
    .await;

    // let result = get_public_address(Ipv4Addr::new(192, 168, 12, 1).into(), None).await;

    println!("{:?}", result);

    Ok(())
}

fn main() -> Result<(), color_eyre::Report> {
    // set up .env
    // dotenv().expect(".env file not found");

    color_eyre::config::HookBuilder::default()
        .capture_span_trace_by_default(false)
        .install()?;

    let rust_log_value = env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| format!("INFO,{}=TRACE", env!("CARGO_PKG_NAME").replace('-', "_")));

    // set up logger
    // from_env defaults to RUST_LOG
    tracing_subscriber::registry()
        .with(EnvFilter::builder().parse(rust_log_value).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default())
        .init();

    // initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // start service
    let result: Result<(), color_eyre::Report> = rt.block_on(start_tasks());

    result
}
