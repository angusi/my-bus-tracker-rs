//! Disruptions Web Service
//!
//! For full documentation, see Section IV.3 of the My Bus Tracker API Guide (Version F)

use hyper::{Method, Request};
use super::{models, MyBusTracker, MyBusTrackerError};
use futures::{self, Future};
use chrono::{Date, Duration, Utc};

/// Disruptions Web Service
///
/// To use methods from the Disruptions Web Service, bring this trait into scope
/// alongside your `MyBusTracker` instance.
#[allow(stutter)]
pub trait DisruptionsServices {
    /// Get a list of ongoing disruptions.
    ///
    /// You may request disruptions of a specific type. If you do not specify a type, all
    /// disruptions are returned.
    fn get_disruptions(
        &self,
        disruption_type: &Option<&models::DisruptionType>,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Disruptions, Error = MyBusTrackerError>>;

    /// Get a list of ongoing diversions.
    ///
    /// You may request disruptions on:
    ///   - optionally, a specific service - the default is all services;
    ///   - optionally, a specific date, up to three-days in the future - the default is today;
    fn get_diversions(
        &self,
        service_reference: &Option<&str>,
        day: &Option<Date<Utc>>,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Diversions, Error = MyBusTrackerError>>;

    /// Get the description of a diversion for plotting on a map
    fn get_diversion_points(
        &self,
        diversion: &str,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::DiversionPoints, Error = MyBusTrackerError>>;
}

impl DisruptionsServices for MyBusTracker {
    fn get_disruptions(
        &self,
        disruption_type: &Option<&models::DisruptionType>,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Disruptions, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting disruptions";
            "type" => ?disruption_type,
            "operator" => ?operator,
        );

        let disruption_type = disruption_type.unwrap_or(&models::DisruptionType::All);

        let uri_params = format!(
            "operatorId={}&type={}",
            operator.to_string(),
            disruption_type
        );
        let uri = match self.get_uri("getDisruptions", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }

    fn get_diversions(
        &self,
        service_reference: &Option<&str>,
        day: &Option<Date<Utc>>,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Diversions, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting diversions";
            "service_reference" => service_reference,
            "day" => ?day,
            "operator" => ?operator,
        );

        let service_reference = service_reference.unwrap_or("0");

        let day_difference: Duration = match *day {
            Some(day) => day.signed_duration_since(Utc::today()),
            None => Duration::days(0),
        };
        if day_difference > Duration::days(3) || day_difference < Duration::days(0) {
            return Box::new(futures::failed(MyBusTrackerError::DateOutOfBounds));
        }

        let uri_params = format!(
            "operatorId={}&refService={}&day={}",
            operator,
            service_reference,
            day_difference.num_days()
        );
        let uri = match self.get_uri("getDiversions", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }
    fn get_diversion_points(
        &self,
        diversion: &str,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::DiversionPoints, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting diversion points";
            "diversion" => diversion,
            "operator" => ?operator,
        );
        let uri_params = format!("operatorId={}&diversionId={}", operator, diversion);
        let uri = match self.get_uri("getDiversionPoints", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }
}
