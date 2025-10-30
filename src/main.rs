mod build_env;
mod utils;

use std::env;
use std::env::VarError;
use std::net::Ipv4Addr;
use std::num::NonZeroU16;

use color_eyre::config::HookBuilder;
use color_eyre::eyre;
use natpmp_rs::protocol::MappingProtocol;
use natpmp_rs::{get_public_address, map_port};
use tracing::{Level, event};
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Layer as _};

use crate::build_env::get_build_env;
use crate::utils::flatten_handle;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn build_filter() -> (EnvFilter, Option<eyre::Report>) {
    fn build_default_filter() -> EnvFilter {
        EnvFilter::builder()
            .parse(format!("INFO,{}=TRACE", env!("CARGO_CRATE_NAME")))
            .expect("Default filter should always work")
    }

    let (filter, parsing_error) = match env::var(EnvFilter::DEFAULT_ENV) {
        Ok(user_directive) => match EnvFilter::builder().parse(user_directive) {
            Ok(filter) => (filter, None),
            Err(error) => (build_default_filter(), Some(eyre::Report::new(error))),
        },
        Err(VarError::NotPresent) => (build_default_filter(), None),
        Err(error @ VarError::NotUnicode(_)) => {
            (build_default_filter(), Some(eyre::Report::new(error)))
        },
    };

    (filter, parsing_error)
}

fn init_tracing(filter: EnvFilter) -> Result<(), eyre::Report> {
    let registry = tracing_subscriber::registry();

    #[cfg(feature = "tokio-console")]
    let registry = registry.with(console_subscriber::ConsoleLayer::builder().spawn());

    Ok(registry
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .with(tracing_error::ErrorLayer::default())
        .try_init()?)
}

fn print_header() {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let build_env = get_build_env();

    event!(
        Level::INFO,
        "{} v{} - built for {} ({})",
        NAME,
        VERSION,
        build_env.get_target(),
        build_env.get_target_cpu().unwrap_or("base cpu variant"),
    );
}

fn main() -> Result<(), color_eyre::Report> {
    // set up .env
    // dotenv().expect(".env file not found");

    HookBuilder::default()
        .capture_span_trace_by_default(true)
        .display_env_section(false)
        .install()?;

    let (env_filter, parsing_error) = build_filter();

    init_tracing(env_filter)?;

    // bubble up the parsing error
    parsing_error.map_or(Ok(()), Err)?;

    // initialize the runtime
    let result: Result<(), eyre::Report> = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async {
            // explicitly launch everything in a spawned task
            // see https://docs.rs/tokio/latest/tokio/attr.main.html#non-worker-async-function
            let handle = tokio::task::spawn(start_tasks());

            flatten_handle(handle).await
        });

    result
}

/// starts all the tasks, such as the web server, the key refresh, ...
/// ensures all tasks are gracefully shutdown in case of error, ctrl+c or sigterm
async fn start_tasks() -> Result<(), color_eyre::Report> {
    print_header();

    let result = map_port(
        MappingProtocol::TCP,
        NonZeroU16::new(9_999).unwrap(),
        // Some(unsafe { NonZeroU16::new_unchecked(10_000) }),
        None,
        None,
        // None,
        Ipv4Addr::new(192, 168, 12, 55).into(),
        None,
    )
    .await;

    match result {
        Ok(ok) => {
            println!("{}", ok);
        },
        Err(error) => {
            println!("{}", error);
        },
    }

    let result = get_public_address(Ipv4Addr::new(192, 168, 12, 1).into(), None).await;

    match result {
        Ok(ok) => {
            println!("{}", ok);
        },
        Err(error) => {
            println!("{}", error);
        },
    }

    Ok(())
}
