//! Tiny outbound HTTP client built directly on hyper + native-tls.
//!
//! Used for the small surface we actually need (Telegram Bot API + a few
//! S3 helpers). Kept dependency-light on purpose.

use anyhow::Result;
use async_compression::tokio::write::{GzipDecoder, GzipEncoder};
use bytes::Bytes;
use futures::Future;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::Body, client::conn::http1::SendRequest, header, header::HeaderValue, http::Method,
    HeaderMap, Request, StatusCode, Uri,
};
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub struct Response {
    pub status: StatusCode,
    data: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode, data: Vec<u8>) -> Self {
        Self { status, data }
    }

    #[inline]
    pub fn to_json<'a, R: serde::de::Deserialize<'a>>(&'a self) -> Result<R> {
        Ok(serde_json::from_slice(&self.data)?)
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.data
    }
}

pub async fn get(uri: &Uri, headers: Option<&HeaderMap>) -> Result<Response> {
    timeout(fetch(Method::GET, uri, headers, Default::default())).await
}

pub async fn post(
    uri: &Uri,
    headers: Option<&HeaderMap>,
    body: impl Into<bytes::Bytes>,
) -> Result<Response> {
    timeout(fetch(Method::POST, uri, headers, body.into())).await
}

async fn timeout<R>(fetch: impl Future<Output = Result<R>>) -> Result<R> {
    const TIMEOUT_DURATION: Duration = Duration::from_millis(60_000);
    tokio::time::timeout(TIMEOUT_DURATION, fetch).await?
}

async fn connect<'a>(
    uri: &'a Uri,
    required_scheme: &str,
    default_port: u16,
) -> Result<(&'a str, TcpStream)> {
    let host = uri
        .host()
        .ok_or_else(|| anyhow::anyhow!("uri missing host: {uri}"))?;
    let scheme = uri.scheme_str();

    if scheme != Some(required_scheme) {
        anyhow::bail!("scheme mismatch: expected {required_scheme}, got {scheme:?}");
    }
    let addr = (host, uri.port_u16().unwrap_or(default_port));
    let stream = TcpStream::connect(&addr).await?;

    Ok((host, stream))
}

async fn dial_tls(uri: &Uri) -> Result<TokioIo<tokio_native_tls::TlsStream<TcpStream>>> {
    let (host, tcp_stream) = connect(uri, "https", 443).await?;
    let cx = tokio_native_tls::native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let cx = tokio_native_tls::TlsConnector::from(cx);
    let stream = cx.connect(host, tcp_stream).await?;
    Ok(TokioIo::new(stream))
}

async fn fetch(
    method: Method,
    uri: &Uri,
    headers: Option<&HeaderMap>,
    body: Bytes,
) -> Result<Response> {
    let io = dial_tls(uri).await?;

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            tracing::warn!(error = ?err, "http connection failed");
        }
    });

    send(method, uri, headers, body, &mut sender).await
}

async fn send(
    method: Method,
    uri: &Uri,
    headers: Option<&HeaderMap>,
    mut body: Bytes,
    sender: &mut SendRequest<Full<Bytes>>,
) -> Result<Response> {
    let mut req = Request::builder().method(&method).uri(uri);

    let headers_mut = req
        .headers_mut()
        .ok_or_else(|| anyhow::anyhow!("request builder lost headers map"))?;
    if let Some(authority) = uri.authority() {
        headers_mut.insert(header::HOST, HeaderValue::from_str(authority.as_str())?);
    }

    if let Some(headers) = headers.as_ref() {
        for (key, value) in headers.iter() {
            headers_mut.insert(key, value.to_owned());
        }
    }

    if !headers_mut.contains_key(header::ACCEPT_ENCODING) {
        headers_mut.insert(header::ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
    }

    let body: Full<Bytes> = if method == Method::POST && body.len() > 1000 {
        match headers_mut
            .get(header::CONTENT_ENCODING)
            .and_then(|h| h.to_str().ok())
        {
            Some("gzip") => {
                let mut encoder = GzipEncoder::new(vec![]);
                encoder.write_all_buf(&mut body).await?;
                encoder.shutdown().await?;
                encoder.into_inner().into()
            }
            Some(other) => {
                tracing::warn!(content_encoding = %other, "unsupported content_encoding for compression");
                body.into()
            }
            None => body.into(),
        }
    } else {
        headers_mut.remove(header::CONTENT_ENCODING);
        body.into()
    };

    // Some APIs (e.g. Telegram, YT) require Content-Length even when empty.
    if let Some(len) = body.size_hint().exact() {
        headers_mut.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    }

    let req = req.body(body)?;

    let mut res = sender.send_request(req).await?;
    let status = res.status();
    match res
        .headers()
        .get(header::CONTENT_ENCODING)
        .and_then(|h| h.to_str().ok())
    {
        Some("gzip") => {
            let mut decoder = GzipDecoder::new(vec![]);
            while let Some(next) = res.frame().await {
                if let Some(chunk) = next?.data_ref() {
                    decoder.write_all(chunk).await?;
                }
            }
            decoder.shutdown().await?;
            Ok(Response::new(status, decoder.into_inner()))
        }
        Some(other) => {
            tracing::warn!(content_encoding = %other, "unsupported response encoding");
            let data = res.collect().await?.to_bytes().into();
            Ok(Response::new(status, data))
        }
        None => {
            let data = res.collect().await?.to_bytes().into();
            Ok(Response::new(status, data))
        }
    }
}
