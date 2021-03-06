use crate::default;
use core::time::Duration;
use std::path::PathBuf;
#[cfg(feature = "rustls")]
use tokio_rustls::rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    Certificate, PrivateKey,
};

/// Ssl struct contains the key and cert
/// required to setup SSL with the selected
/// feature (rustls or openssl)
///
/// By default, the Config struct has an ssl field so
/// you don't have to use it directly but if you want
/// to then you can and then append it to the config by
/// [`with_ssl_config()`]()
///
/// ```no_run
/// use octane::Octane;
/// use std::time::Duration;
/// use octane::config::{Config, Ssl};
///
/// let mut app = Octane::new();
/// let mut ssl_config = Ssl::new();
/// ssl_config.key("templates/key.pem");
/// app.with_ssl_config(ssl_config);
/// ```
#[derive(Clone)]
pub struct Ssl {
    /// Location of the private key file, should be an `RSA_PRIVATE_KEY`.
    pub key: PathBuf,
    /// Location of the certificate file.
    pub cert: PathBuf,
    /// The port where TLS should listen, it is 443 by default.
    pub port: u16,
}

impl Ssl {
    /// Returns a new Ssl struct instance with default port
    /// 443
    pub fn new() -> Self {
        Ssl {
            key: PathBuf::new(),
            cert: PathBuf::new(),
            port: 443,
        }
    }
    /// Mutates the Ssl struct and sets the private key path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use octane::config::OctaneConfig;
    ///
    /// let mut config = OctaneConfig::new();
    /// config
    ///    .ssl
    ///    .key("templates/key.pem");
    /// ```
    pub fn key(&mut self, path: &str) -> &mut Self {
        self.key = PathBuf::from(path);
        self
    }
    /// Mutates the Ssl struct and sets the SSL certificate path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use octane::config::OctaneConfig;
    ///
    /// let mut config = OctaneConfig::new();
    /// config
    ///    .ssl
    ///    .cert("templates/cert.pem");
    /// ```
    pub fn cert(&mut self, path: &str) -> &mut Self {
        self.cert = PathBuf::from(path);
        self
    }
}

/// An independent OctaneConfig struct that can be used
/// separately from the app structure and then be appended
/// to it.
///
/// **Note**: If you won't push the independently made config
/// then the configurations won't take place, make sure
/// to push them to the main server struct like the following
///
/// ```no_run
/// use octane::Octane;
/// use octane::config::OctaneConfig;
///
/// let mut app = Octane::new();
/// let mut config = OctaneConfig::new();
/// app.with_config(config);
/// ```
///
/// The config holds the values for various configurable
/// item. If no config is specified then defaults are used.
///
pub struct OctaneConfig {
    /// The duration for keep alive requests. It is 5 seconds by default
    pub keep_alive: Option<Duration>,
    /// An instance of the `Ssl` struct to store the values of key and certificates.
    pub ssl: Ssl,
    worker_threads: Option<usize>,
}

/// Shared config trait which allows us to use the config
/// methods on the Octane server struct too as it has a
/// config field by default
pub trait Config {
    /// Sets the keepalive duration for a keepalive request,
    /// by default, a 5 second keep alive is set
    /// # Example
    ///
    /// ```no_run
    /// use octane::config::{OctaneConfig, Config};
    /// use std::time::Duration;
    ///
    /// let mut config = OctaneConfig::new();
    /// config.set_keepalive(Duration::new(5, 0));
    /// ```
    ///
    /// Or with Octane struct
    ///
    /// ```no_run
    /// use octane::Octane;
    /// use std::time::Duration;
    /// use octane::config::Config;
    ///
    /// let mut app = Octane::new();
    /// app.set_keepalive(Duration::new(5, 0));
    /// ```
    fn set_keepalive(&mut self, duration: Duration);
    /// Replaces the current ssl config with the one
    /// specified in the arguments
    ///
    /// # Example
    ///
    /// ```no_run
    /// use octane::config::{OctaneConfig, Config, Ssl};
    /// use std::time::Duration;
    ///
    /// let mut config = OctaneConfig::new();
    /// let mut ssl_config = Ssl::new();
    /// ssl_config.key("templates/key.pem");
    /// ```
    ///
    /// Or with Octane struct
    ///
    /// ```no_run
    /// use octane::Octane;
    /// use std::time::Duration;
    /// use octane::config::{Config, Ssl};
    ///
    /// let mut app = Octane::new();
    /// let mut ssl_config = Ssl::new();
    /// ssl_config.key("templates/key.pem");
    /// app.with_ssl_config(ssl_config);
    /// ```
    fn with_ssl_config(&mut self, ssl_conf: Ssl);
    /// Returns the Ssl instance of the config and
    /// sets the port number for TLS
    ///
    /// # Example
    ///
    /// ```no_run
    /// use octane::config::{OctaneConfig, Config};
    /// use std::time::Duration;
    ///
    /// let mut config = OctaneConfig::new();
    /// config
    ///     .ssl(443)
    ///     .key("key.pem")
    ///     .cert("cert.pem");
    /// ```
    ///
    /// Or with Octane struct
    ///
    /// ```no_run
    /// use octane::Octane;
    /// use std::time::Duration;
    /// use octane::config::Config;
    ///
    /// let mut app = Octane::new();
    /// app
    ///     .ssl(443)
    ///     .key("key.pem")
    ///     .cert("cert.pem");
    /// ```
    fn ssl(&mut self, port: u16) -> &mut Ssl;
}
/// Octane config which can be used independently to
/// configure the server settings.
///
/// To apply the settings to the main struct,
/// make sure you run `app.with_config(config);` on the
/// main struct where config is the custom config you created
impl OctaneConfig {
    /// Creates a new config instance with default values
    pub fn new() -> Self {
        OctaneConfig {
            ssl: Ssl::new(),
            keep_alive: Some(Duration::from_secs(5)),
            worker_threads: None,
        }
    }
    // Appends a settings instance to self
    pub(crate) fn append(&mut self, settings: Self) {
        self.ssl = settings.ssl;
        self.keep_alive = settings.keep_alive;
    }

    /// Sets the number of worker threads, this is settings
    /// which will be applied to the `core_threads` method
    /// on the runtime builder struct. Alias for
    /// [tokio's core_threads](https://docs.rs/tokio/0.2.13/tokio/runtime/struct.Builder.html#method.core_threads)
    pub fn worker_threads(&mut self, threads: usize) -> &mut Self {
        self.worker_threads = Some(threads);
        self
    }

    // Get the certs as a Vec<Certificate>, a user will not have to
    // use this directly, this is used and done for them
    #[cfg(feature = "rustls")]
    pub(crate) fn get_cert(&self) -> std::io::Result<Vec<Certificate>> {
        let mut buf = std::io::BufReader::new(std::fs::File::open(&self.ssl.cert)?);
        certs(&mut buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid Certs"))
    }
    // Get the private key as a Vec<PrivateKey>, a user will not have to
    // use this directly
    #[cfg(feature = "rustls")]
    pub(crate) fn get_key(&self) -> std::io::Result<Vec<PrivateKey>> {
        let mut buf = std::io::BufReader::new(std::fs::File::open(&self.ssl.key)?);
        rsa_private_keys(&mut buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid Key"))
    }
}

default!(OctaneConfig);
default!(Ssl);

impl Config for OctaneConfig {
    fn set_keepalive(&mut self, duration: Duration) {
        self.keep_alive = Some(duration);
    }

    fn with_ssl_config(&mut self, ssl_conf: Ssl) {
        self.ssl.key = ssl_conf.key;
        self.ssl.cert = ssl_conf.cert;
    }
    fn ssl(&mut self, port: u16) -> &mut Ssl {
        self.ssl.port = port;
        &mut self.ssl
    }
}
