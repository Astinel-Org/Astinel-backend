use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(false)
        .compact()
        .init();
}

pub fn init_quiet() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("error"))
        .with_target(false)
        .compact()
        .init();
}
