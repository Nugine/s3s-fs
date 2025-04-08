use s3s_fs::Result;

use std::ops::Not;

use s3s::auth::SecretKey;
use s3s::auth::SimpleAuth;
use s3s::host::MultiDomain;
use s3s::service::S3Service;
use s3s::service::S3ServiceBuilder;

use camino::Utf8PathBuf;
use clap::Parser;
use tracing::{error, info};

#[derive(Debug, Parser)]
#[command(version)]
struct Opt {
    /// Root directory of all internal data.
    #[arg(long)]
    root: Utf8PathBuf,

    /// Access key used for authentication.
    #[arg(long)]
    access_key: Option<String>,

    /// Secret key used for authentication.
    #[arg(long)]
    secret_key: Option<SecretKey>,

    /// Host name to listen on.
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Port number to listen on.
    #[arg(long, default_value = "8014")] // The original design was finished on 2020-08-14.
    port: u16,

    /// Domain names used for virtual-hosted-style requests.
    #[arg(long)]
    domains: Vec<String>,
}

fn parse_cli_args() -> Opt {
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    let mut opt = Opt::parse();
    info!(?opt);

    let mut cmd = Opt::command();

    if let (Some(_), None) | (None, Some(_)) = (&opt.access_key, &opt.secret_key) {
        let msg = "access key and secret key must be specified together";
        cmd.error(ErrorKind::MissingRequiredArgument, msg).exit();
    }

    for s in &opt.domains {
        if s.contains('/') {
            let msg = format!("expected domain name, found URL-like string: {s:?}");
            cmd.error(ErrorKind::InvalidValue, msg).exit();
        }
    }

    let Ok(root) = opt.root.canonicalize_utf8().map_err(|e| {
        let msg = format!("failed to canonicalize root directory {:?}: {:?}", opt.root, e);
        cmd.error(ErrorKind::ValueValidation, msg).exit()
    });

    // switch to root directory for shorter paths
    match std::env::set_current_dir(&root) {
        Ok(()) => {
            opt.root = Utf8PathBuf::from(".");
        }
        Err(e) => {
            let msg = format!("failed to set current directory to {root:?}: {e:?}");
            cmd.error(ErrorKind::ValueValidation, msg).exit();
        }
    }

    opt
}

fn setup_tracing() {
    use std::io::IsTerminal;
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::from_default_env();
    let enable_color = std::io::stdout().is_terminal();

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(env_filter)
        .with_ansi(enable_color)
        .init();
}

fn build_s3_service(opt: &Opt, system: s3s_fs::System) -> Result<S3Service> {
    let mut b = S3ServiceBuilder::new(system);

    // Enable authentication
    if let (Some(ak), Some(sk)) = (opt.access_key.clone(), opt.secret_key.clone()) {
        b.set_auth(SimpleAuth::from_single(ak, sk));
    }

    // Enable parsing virtual-hosted-style requests
    if opt.domains.is_empty().not() {
        b.set_host(MultiDomain::new(&opt.domains)?);
    }

    Ok(b.build())
}

async fn serve(opt: &Opt, service: S3Service) -> Result<()> {
    let listener = tokio::net::TcpListener::bind((opt.host.as_str(), opt.port)).await?;
    let local_addr = listener.local_addr()?;

    let http = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new());
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

    let mut signal = std::pin::pin!(async {
        tokio::signal::ctrl_c()
            .await
            .expect("CTRL+C signal handler should not fail");
    });

    info!("http server is running at http://{local_addr}");

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        let io = hyper_util::rt::TokioIo::new(stream);
                        let conn = http.serve_connection(io, service.clone());
                        let fut = graceful.watch(conn.into_owned());
                        tokio::spawn(async move {
                            if let Err(e) = fut.await {
                                error!("error serving connection: {e}");
                            }
                        });
                    },
                    Err(err) => {
                        error!("error accepting connection: {err}");
                    }
                }
            },
            () = signal.as_mut() => {
                info!("graceful shutdown signal received");
                break;
            }
        }
    }

    info!("shutting down all connections");
    graceful.shutdown().await;

    info!("http server is stopped");

    Ok(())
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    let opt = parse_cli_args();

    setup_tracing();

    let system = s3s_fs::System::new(&opt.root).await?;

    let service = build_s3_service(&opt, system.clone())?;

    serve(&opt, service).await?;

    system.shutdown().await?;

    Ok(())
}
