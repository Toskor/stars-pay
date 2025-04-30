// This file is needed to get rid of dependencies from yoml and cidre.
// Mb later this dublicated code will be removed.

use anyhow::Result;
use async_compression::tokio::write::{GzipDecoder, GzipEncoder};
use bytes::Bytes;
use futures::Future;
use http_body_util::{BodyExt, Full};
use hyper::{body::Body, client::conn::http1::SendRequest, Request, StatusCode};
use hyper::{header, header::HeaderValue, http::Method, HeaderMap, Uri};
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::{fs::File, io::AsyncWriteExt, net::TcpStream};

pub struct Response {
    pub status: StatusCode,
    data: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode, data: Vec<u8>) -> Self {
        Self { status, data }
    }

    pub fn to_str(&self) -> Result<&str> {
        Ok(std::str::from_utf8(&self.data)?)
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

pub async fn delete(uri: &Uri, headers: Option<&HeaderMap>) -> Result<Response> {
    timeout(fetch(Method::DELETE, uri, headers, Default::default())).await
}

pub async fn put(
    uri: &Uri,
    headers: Option<&HeaderMap>,
    body: impl Into<bytes::Bytes>,
) -> Result<Response> {
    timeout(fetch(Method::PUT, uri, headers, body.into())).await
}

pub async fn fetch_file(uri: &Uri, path: &str) -> Result<()> {
    timeout(fetch_data(uri, path)).await
}

async fn timeout<R>(fetch: impl Future<Output = Result<R>>) -> Result<R> {
    const TIMEOUT_DURATION: Duration = Duration::from_millis(60_000);

    match tokio::time::timeout(TIMEOUT_DURATION, fetch).await {
        Ok(result) => match result {
            Err(e) => Err(e)?,
            ok => ok,
        },
        err => err?,
    }
}

async fn connect<'a>(
    uri: &'a hyper::Uri,
    required_scheme: &str,
    default_port: u16,
) -> Result<(&'a str, TcpStream)> {
    let host = uri.host().unwrap();
    let scheme = uri.scheme_str();
    // todo port dependent on scheme http 80 https 443
    let port = uri.port_u16();

    if scheme != Some(required_scheme) {
        anyhow::bail!("schemes don't match")
    }
    let addr = (host, port.unwrap_or(default_port));
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

pub struct Client {
    sender: Option<SendRequest<Full<Bytes>>>,
}

impl Client {
    pub fn new() -> Self {
        Self { sender: None }
    }

    async fn fetch(
        &mut self,
        method: Method,
        uri: &Uri,
        headers: Option<&HeaderMap>,
        body: Bytes,
    ) -> Result<Response> {
        let sender = self.sender(uri).await?;
        send(method, uri, headers, body, sender).await
    }

    async fn sender(&mut self, uri: &Uri) -> Result<&mut SendRequest<Full<Bytes>>> {
        if !self.sender.as_ref().map_or(true, |s| s.is_closed()) {
            return Ok(unsafe { self.sender.as_mut().unwrap_unchecked() });
        }

        let io = dial_tls(uri).await?;

        let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        self.sender = Some(sender);
        Ok(unsafe { self.sender.as_mut().unwrap_unchecked() })
    }

    pub async fn get(&mut self, uri: &Uri, headers: Option<&HeaderMap>) -> Result<Response> {
        timeout(self.fetch(Method::GET, uri, headers, Bytes::new())).await
    }

    pub async fn post(
        &mut self,
        uri: &Uri,
        headers: Option<&HeaderMap>,
        body: impl Into<bytes::Bytes>,
    ) -> Result<Response> {
        timeout(self.fetch(Method::POST, uri, headers, body.into())).await
    }
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
            println!("Connection failed: {:?}", err);
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

    let headers_mut = req.headers_mut().unwrap();
    if let Some(authority) = uri.authority() {
        headers_mut.insert(
            header::HOST,
            HeaderValue::from_str(authority.as_str()).unwrap(),
        );
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
            // Some("br") => {
            //     let mut encoder = BrotliEncoder::new(vec![]);
            //     encoder.write_all_buf(&mut body).await?;
            //     encoder.shutdown().await?;
            //     encoder.into_inner().into()
            // }
            Some(_) => {
                println!("unsuported content_encoding compress");
                debug_assert!(false);
                body.into()
            }
            None => body.into(),
        }
    } else {
        headers_mut.remove(header::CONTENT_ENCODING);
        body.into()
    };

    if let Some(len) = body.size_hint().exact() {
        //note: yt api requires header::CONTENT_LENGTH even if its len is 0
        if len >= 0 {
            headers_mut.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_bytes(&len.to_string().into_bytes()).unwrap(),
            );
        }
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
                    decoder.write_all(&chunk).await?;
                }
            }
            decoder.shutdown().await?;
            Ok(Response::new(status, decoder.into_inner()))
        }
        // Some("br") => {
        //     let mut decoder = BrotliDecoder::new(vec![]);
        //     while let Some(next) = res.frame().await {
        //         if let Some(chunk) = next?.data_ref() {
        //             decoder.write_all(&chunk).await?;
        //         }
        //     }
        //     decoder.shutdown().await?;
        //     Ok(Response::new(status, decoder.into_inner()))
        // }
        Some(wtf) => {
            eprintln!("unsupported content type: {wtf}");
            debug_assert!(false);
            let data = res.collect().await?.to_bytes().into();
            Ok(Response::new(status, data))
        }
        None => {
            let data = res.collect().await?.to_bytes().into();
            Ok(Response::new(status, data))
        }
    }
}

async fn fetch_data(uri: &Uri, path: &str) -> Result<()> {
    let io = dial_tls(uri).await?;

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = uri.authority().unwrap();

    let req = Request::get(uri)
        .header(header::HOST, authority.as_str())
        .body(String::new())?;

    let mut res = sender.send_request(req).await?;

    let mut file = File::create(path).await?;

    while let Some(next) = res.frame().await {
        if let Some(chunk) = next?.data_ref() {
            file.write_all(chunk).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::http;
    use hyper::{
        header::{self, HeaderName, HeaderValue},
        HeaderMap,
    };
    use std::path::Path;

    #[test]
    fn headers() {
        let client_id_key = HeaderName::from_bytes(&*b"Client-Id").unwrap();
        let headers = HeaderMap::from_iter([
            (header::ACCEPT, "application/json"),
            (client_id_key, "c2982d16782612932d747554eeb8816b"),
            (header::CONTENT_ENCODING, "gzip"),
        ]);
        assert_eq!(headers.len(), 3)
    }

    #[tokio::test]
    async fn fetch() {
        let headers = HeaderMap::from_iter([
            (
                header::ACCEPT,
                HeaderValue::from_bytes("Application/json".as_bytes()).unwrap(),
            ),
            (
                HeaderName::from_bytes("Client-Id".as_bytes()).unwrap(),
                HeaderValue::from_bytes("C2982d16782612932d747554eeb8816b".as_bytes()).unwrap(),
            ),
            (header::CONTENT_ENCODING, HeaderValue::from_static("gzip")),
        ]);

        let _url_gzip = http::Uri::from_static("https://httpbin.org/gzip");
        let _url_post = http::Uri::from_static("https://httpbin.org/post");
        let url_anything = http::Uri::from_static("https://httpbin.org/anything");
        let _url_trovo =
            http::Uri::from_static("https://open-api.trovo.live/openplatform/getusers");

        let body = r#"{"user": ["torsor"], "id": "123459996"}"#;

        let res_gzip = http::fetch(
            http::Method::GET,
            &url_anything,
            Some(&headers),
            body.into(),
        )
        .await
        .and_then(|r| Ok(r.to_str().unwrap().to_string()))
        .unwrap();
        println!("{res_gzip}");
        let _res = http::fetch(
            http::Method::GET,
            &url_anything,
            Some(&headers),
            body.into(),
        )
        .await
        .and_then(|r| Ok(r.to_str().unwrap().to_string()))
        .unwrap();
    }

    #[tokio::test]
    async fn file_test() {
        let path = "test_data/emote.png";
        let uri = hyper::Uri::from_static(
            "https://static-cdn.jtvnw.net/emoticons/v2/emotesv2_4c3b4ed516de493bbcd2df2f5d450f49/animated/dark/1.0",
        );

        http::fetch_data(&uri, &path).await.unwrap();

        if !Path::new(&path).exists() {
            assert!(false);
        }
    }
    #[tokio::test]
    async fn http() {
        let uri = hyper::Uri::from_static("https://httpbin.org/anything");
        let headers = HeaderMap::from_iter([
            (header::ACCEPT, HeaderValue::from_static("application/json")),
            (
                HeaderName::from_bytes(b"Client-Id").unwrap(),
                HeaderValue::from_static("c2982d16782612932d747554eeb8816b"),
            ),
            (header::CONTENT_ENCODING, HeaderValue::from_static("gzip")),
        ]);

        http::get(&uri, Some(&headers))
            .await
            .and_then(|r| Ok(r.to_str().unwrap().to_string()))
            .expect("failed to get");

        let body = "body";
        http::post(&uri, Some(&headers), body)
            .await
            .and_then(|r| Ok(r.to_str().unwrap().to_string()))
            .expect("failed to post");
    }
}
