use std::future::Future;

pub fn make_shutdown_signal() -> impl Future<Output = ()> {
    use tokio::signal;
    use tokio::signal::unix::SignalKind;
    #[cfg(unix)]
    let shutdown = async {
        let mut terminate_signal = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            _ = terminate_signal.recv() => { },
            _ = signal::ctrl_c() => { },
        }
    };
    #[cfg(not(unix))]
    let shutdown = async {
        signal::ctrl_c().await.unwrap();
    };
    shutdown
}
