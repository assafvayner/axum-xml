use std::net::SocketAddr;
use std::{assert_eq, println};

use axum::routing::post;
use axum::Router;
use http::StatusCode;
use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::Xml;

pub struct TestClient {
    client: reqwest::Client,
    addr: SocketAddr,
}

impl TestClient {
    pub(crate) async fn new(app: Router) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();
        println!("Listening on {}", addr);

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        // Give the server a moment to start up
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Self {
            client: reqwest::Client::new(),
            addr,
        }
    }

    pub(crate) fn post(&self, url: &str) -> RequestBuilder {
        self.client.post(format!("http://{}{}", self.addr, url))
    }
}

#[tokio::test]
async fn deserialize_body() {
    #[derive(Debug, Deserialize)]
    struct Input {
        foo: String,
    }

    async fn handler(input: Xml<Input>) -> String {
        input.0.foo
    }

    let app = Router::new().route("/", post(handler));

    let client = TestClient::new(app).await;
    let res = client
        .post("/")
        .body(r#"<Input><foo>bar</foo></Input>"#)
        .header("content-type", "application/xml")
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();

    assert_eq!(body, "bar");
}

#[tokio::test]
async fn consume_body_to_xml_requires_xml_content_type() {
    #[derive(Debug, Deserialize)]
    struct Input {
        foo: String,
    }

    async fn handler(input: Xml<Input>) -> String {
        input.0.foo
    }

    let app = Router::new().route("/", post(handler));

    let client = TestClient::new(app).await;
    let res = client
        .post("/")
        .body(r#"<Input><foo>bar</foo></Input>"#)
        .send()
        .await
        .unwrap();

    let status = res.status();
    assert!(res.text().await.is_ok());

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn xml_content_types() {
    async fn valid_xml_content_type(content_type: &str) -> bool {
        #[derive(Deserialize)]
        struct Value {}

        println!("testing {:?}", content_type);

        async fn handler(Xml(_): Xml<Value>) {}

        let app = Router::new().route("/", post(handler));

        let res = TestClient::new(app)
            .await
            .post("/")
            .header("content-type", content_type)
            .body("<Value />")
            .send()
            .await
            .unwrap();

        res.status() == StatusCode::OK
    }

    assert!(valid_xml_content_type("application/xml").await);
    assert!(valid_xml_content_type("application/xml; charset=utf-8").await);
    assert!(valid_xml_content_type("application/xml;charset=utf-8").await);
    assert!(valid_xml_content_type("application/cloudevents+xml").await);
    assert!(valid_xml_content_type("text/xml").await);
    assert!(!valid_xml_content_type("application/json").await);
}
