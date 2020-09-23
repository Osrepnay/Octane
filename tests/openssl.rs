#![cfg(feature = "openSSL")]
use curl::easy::Easy;
use octane::prelude::*;
use reqwest::ClientBuilder;
mod common;

#[test]
pub fn basic_https_hello_world_openssl() {
    let mut app = Octane::new();
    app.ssl(8000)
        .key("templates/key.pem")
        .cert("templates/cert.pem");
    let string = "Hello, World";
    app.get(
        "/",
        route_next!(|req, res| {
            res.send(string);
        }),
    )
    .unwrap();
    common::run(app, || async {
        let mut easy = Easy::new();
        easy.url(&path!("")).unwrap();

        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                assert_eq!(data, string.as_bytes());
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    })
}
