use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::net::SocketAddr;
use tracing::{info, error};

use crate::metrics::render_prometheus;

async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/metrics" => {
            let body = render_prometheus();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "text/plain; version=0.0.4")
                .body(Body::from(body))
                .unwrap())
        }
        "/health" => {
            Ok(Response::builder()
                .status(200)
                .body(Body::from("ok"))
                .unwrap())
        }
        _ => {
            Ok(Response::builder()
                .status(404)
                .body(Body::from("Not found. Try /metrics or /health"))
                .unwrap())
        }
    }
}

pub async fn start_prometheus_server(port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Prometheus metrics available at http://0.0.0.0:{}/metrics", port);

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        error!("Prometheus server error: {}", e);
    }
}
