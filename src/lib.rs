//! My Bus Tracker
//!
//! `my_bus_tracker` is a client library for interacting with the My Bus Tracker realtime
//! transit information service provided by the City of Edinburgh Council.
//!
//! This development of this crate is not endorsed by or affiliated with City of Edinburgh Council,
//! Lothian Buses or Ineo Systrans. For the full web API guide, and to request an API key,
//! visit <http://www.mybustracker.co.uk/?page=API%20Key>

extern crate chrono;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate md5;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate tokio_core;
extern crate url;

// Tokio/Future Imports
use futures::{Future, Stream};
use futures::future::ok;
use tokio_core::reactor::Handle;

// Hyper Imports
use hyper::Uri;
use hyper::client::{Client, HttpConnector, Request};
use hyper::header::UserAgent;
// TODO: TLS Support - if MyBusTracker WS supports it
//#[cfg(feature = "rustls")]
//use hyper_rustls::HttpsConnector;
//#[cfg(feature = "rust-native-tls")]
//use hyper_tls;
//#[cfg(feature = "rust-native-tls")]
//type HttpsConnector = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

use std::rc::Rc;
use std::cell::RefCell;

use chrono::prelude::*;
use failure::Error;
use slog::Logger;
use url::Url;

pub mod models;
mod disruptions;
mod topological;
mod bustimes;

pub use disruptions::DisruptionsServices;
pub use topological::TopologicalServices;
pub use bustimes::BusTimesService;

use hyper::error::UriError;

const APP_NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");
const APP_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Errors that can be raised by `MyBusTracker`
#[derive(Debug, Fail)]
pub enum MyBusTrackerError {
    #[fail(display = "Internal error")]
    InternalError { cause: String },
    #[fail(display = "Error communicating with MyBusTracker")]
    CommunicationError { cause: String },
    #[fail(display = "Date out of bounds")]
    DateOutOfBounds,
    #[fail(display = "Too many timetables requested")]
    TooManyTimetables,
    #[fail(display = "Too many departures requested")]
    TooManyDepartures,
}

/// Instance of the My Bus Tracker API.
///
/// Typically, one instance of this struct will be instantiated for your entire application.
pub struct MyBusTracker {
    api_key: RefCell<ApiKey>,
    logger: Logger,
    client: Rc<Client<HttpConnector>>,
    root_url: Url,
}

/// Holds an API Key for accessing the My Bus Tracker Web Service.
///
/// Note that the raw API key, as owned by the developer, is _not_ the API key used to access
/// the API! Instead, "for security", a modified form of that key is used - that is the key
/// returned by the `get_key` method of this struct.
struct ApiKey {
    raw_api_key: String,
    key: String,
    generated: chrono::DateTime<Utc>,
    logger: Logger,
}

impl ApiKey {
    /// Create a new API key representation.
    pub fn new(api_key: &str, logger: &Logger) -> Self {
        trace!(logger, "Instantiating new API Key"; "api_key" => api_key);

        let (key, generated) = generate_api_key(logger, api_key);
        Self {
            raw_api_key: api_key.to_owned(),
            key,
            generated,
            logger: logger.clone(),
        }
    }

    /// Retrieve a valid key.
    ///
    /// Note that a new key should be generated using this method prior to each API request,
    /// as generated API keys may be time- or request-bounded.
    ///
    /// System time must be correct for this function to return valid API keys.
    pub fn get_key(&mut self) -> String {
        trace!(self.logger, "Retrieving current API Key");
        // Per the MyBusTracker WS API Guide (Version F), the generated API key is formed by:
        //   - Concatenating the developer API key and the current UTC time in YYYYMMDDHH format
        //   - Computing the MD5 hash of the concatenated string.
        // That means API keys are only valid for the current hour, and the system time must be
        // accurate. We only need to recalculate the key if the hour has changed since the last
        // request.
        if self.generated.hour() == Utc::now().hour() {
            trace!(
                self.logger,
                "Skipping API Key regeneration as time hasn't shifted enough"
            );
        } else {
            let (key, generated) = generate_api_key(&self.logger, &self.raw_api_key);
            self.key = key;
            self.generated = generated;
        }
        self.key.to_owned()
    }
}

impl MyBusTracker {
    /// Create a new MyBusTracker instance.
    ///
    /// Requires an instance of a logger, your developer API key, and a Tokio handle with which
    /// HTTP API requests will be made.
    pub fn new(logger: &Logger, api_key: &str, handle: &Handle) -> Result<Self, Error> {
        trace!(logger, "Instantiating new MyBusTracker"; "api_key" => api_key);
        let client = Client::configure().build(handle);

        let root_url = Url::parse("http://ws.mybustracker.co.uk/?module=json")?;

        Ok(Self {
            api_key: RefCell::new(ApiKey::new(api_key, logger)),
            logger: logger.clone(),
            client: Rc::new(client),
            root_url,
        })
    }

    /// Return the URI to hit for the given API function with the given URL parameters.
    ///
    /// If the URL parameters are specified, they must already be encoded as URI parameters
    /// (i.e. URL encoded key=value format, and separated with ampersands)
    fn get_uri(&self, function: &str, uri_params: Option<&str>) -> Result<Uri, MyBusTrackerError> {
        trace!(self.logger, "Figuring out URI"; "function" => function, "params" => ?uri_params);
        let api_key = self.api_key.borrow_mut().get_key();
        let merged_params = match uri_params {
            None => format!("key={}&function={}", api_key, function),
            Some(params) => format!("key={}&function={}&{}", api_key, function, params),
        };

        let query = self.root_url.query();
        let query_string = match query {
            None => merged_params,
            Some(query_string) => format!("{}&{}", query_string, merged_params),
        };

        let mut uri = self.root_url.clone();
        uri.set_query(Some(&query_string));
        uri.into_string()
            .parse()
            .map_err(|e: UriError| MyBusTrackerError::InternalError {
                cause: e.to_string(),
            })
    }

    /// Performs the given HTTP request, deserializing the result into the requested type `T`.
    fn make_request<T: 'static>(
        &self,
        mut request: Request,
    ) -> Box<Future<Item = T, Error = MyBusTrackerError>>
    where
        T: serde::de::DeserializeOwned,
    {
        trace!(self.logger, "Performing HTTP request"; "uri" => ?request.uri());

        let client = self.client.clone();

        let useragent_header = UserAgent::new(format!(
            "{}/{}",
            APP_NAME.unwrap_or("my_bus_tracker_rs"),
            APP_VERSION.unwrap_or("unknown")
        ));
        request.headers_mut().set(useragent_header);

        Box::new(
            client
                .request(request)
                .map_err(|e| MyBusTrackerError::CommunicationError {
                    cause: e.to_string(),
                })
                .and_then(|res| {
                    res.body()
                        .fold(Vec::new(), |mut v, chunk| {
                            v.extend(&chunk[..]);
                            ok::<_, hyper::Error>(v)
                        })
                        .map_err(|e| MyBusTrackerError::InternalError {
                            cause: e.to_string(),
                        })
                })
                .and_then(move |chunks| {
                    serde_json::from_slice(&chunks).map_err(|e| MyBusTrackerError::InternalError {
                        cause: e.to_string(),
                    })
                }),
        )
    }
}

/// Take a base API key and turn it into a My Bus Tracker API key, valid for the clock-hour.
fn generate_api_key(logger: &Logger, base_key: &str) -> (String, chrono::DateTime<Utc>) {
    debug!(logger, "Generating API key"; "base_key" => base_key);

    // Per the MyBusTracker WS API Guide (Version F), the generated API key is formed by:
    //   - Concatenating the developer API key and the current UTC time in YYYYMMDDHH format
    //   - Computing the MD5 hash of the concatenated string.
    // That means API keys are only valid for the current hour, and the system time must be
    // accurate.
    let time = Utc::now();
    let time_string = time.format("%Y%m%d%H");

    let raw_key = format!("{}{}", base_key, time_string);

    let computed_key = md5::compute(raw_key);
    let computed_key_string = format!("{:x}", computed_key);

    trace!(logger, "Computed API Key";
           "base_key" => base_key, "time" => %time_string, "computed_key" => %computed_key_string);
    (computed_key_string, time)
}
