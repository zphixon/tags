use anyhow::Context as AnyhowContext;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router, http::StatusCode,
};
use std::{fmt::Display, net::SocketAddr, sync::OnceLock};

mod tags;

#[derive(Debug, serde::Deserialize)]
#[serde(default)]
struct Config {
    address: SocketAddr,
    debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: "0.0.0.0:8888".parse().unwrap(),
            debug: false,
        }
    }
}

pub(crate) static CONFIG: OnceLock<Config> = OnceLock::new();

#[macro_export]
macro_rules! error {
    ($($args:expr),*) => {
        {
            let message = format!("{}:{} {}", file!(), line!(), format_args!($($args),*));
            ::tracing::error!("{}", message);
            ::anyhow::Result::<_>::Err(::anyhow::anyhow!(message))
        }
    };
}

struct Context {
    message: String,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    fn from(d: impl Display) -> Context {
        let message = format!("{}", d);
        Context { message }
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _: anyhow::Result<()> = error!("{}", self.message);
        write!(f, "{}", self.message)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Hello, notetakers!");

    let args = std::env::args().collect::<Vec<_>>();
    match &args[..] {
        [_, option, value] => match (option.as_str(), value.as_str()) {
            ("--config", filename) => {
                let config_string = std::fs::read_to_string(filename)
                    .context(Context::from("Could not read config file"))?;
                let config = serde_json::from_str(&config_string)
                    .context(Context::from("Could not parse config JSON"))?;
                CONFIG.get_or_init(|| config);
            }

            (option, _) => {
                return error!("Unknown option {}", option);
            }
        },

        [_] => {
            CONFIG.get_or_init(Config::default);
        }

        _ => return error!("Only expected --config <filename>"),
    }

    let config = CONFIG.get().expect("config must be set");

    let mut router = Router::new().route("/", get(root)).fallback(not_found);
    if config.debug {
        tracing::debug!("Debugging enabled on /debug");
        router = router.route("/debug", get(debug));
    }

    axum::Server::bind(&config.address)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not found")
}

async fn root() -> impl IntoResponse {
    response::Html("Hello, notetakers!")
}

async fn debug() -> impl IntoResponse {
    response::Html(format!("<pre>{:#?}</pre>", CONFIG.get()))
}
