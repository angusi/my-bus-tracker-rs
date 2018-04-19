//! Topological Web Service
//!
//! For full documentation, see Section IV.2 of the My Bus Tracker API Guide (Version F)

use hyper::{Method, Request};
use super::{models, MyBusTracker, MyBusTrackerError};
use futures::{self, Future};

/// Topological Web Service
///
/// To use methods from the Bus Times Web Service, bring this trait into scope
/// alongside your `MyBusTracker` instance.
#[allow(stutter)]
pub trait TopologicalServices {
    /// Get the ID of the topology version in use.
    ///
    /// This ID is generated only once per day server-side. It is not cached, so subsequent
    /// calls of this function will result in repeated API calls.
    /// The value is only updated if the topology has changed.
    fn get_topo_id(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::TopoId, Error = MyBusTrackerError>>;

    /// Get a list of services in operation.
    fn get_services(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Services, Error = MyBusTrackerError>>;

    /// Get a description of a service route for plotting on a map
    fn get_service_points(
        &self,
        service_reference: &str,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::ServicePoints, Error = MyBusTrackerError>>;

    /// Get a list of service destinations
    fn get_destinations(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Destinations, Error = MyBusTrackerError>>;

    /// Get a list of bus stops
    fn get_bus_stops(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::BusStops, Error = MyBusTrackerError>>;
}

impl TopologicalServices for MyBusTracker {
    fn get_topo_id(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::TopoId, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting topography ID;";
            "operator" => ?operator,
        );
        let uri_params = format!("operatorId={}", operator.to_string());
        let uri = match self.get_uri("getTopoId", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }

    fn get_services(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Services, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting services";
            "operator" => ?operator
        );
        let uri_params = format!("operatorId={}", operator.to_string());
        let uri = match self.get_uri("getServices", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }

    fn get_service_points(
        &self,
        service_reference: &str,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::ServicePoints, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting service points";
            "service_reference" => service_reference,
            "operator" => ?operator,
        );
        let uri_params = format!(
            "operatorId={}&ref={}",
            operator.to_string(),
            service_reference
        );
        let uri = match self.get_uri("getServicePoints", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }

    fn get_destinations(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::Destinations, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting destinations";
            "operator" => ?operator
        );
        let uri_params = format!("operatorId={}", operator.to_string());
        let uri = match self.get_uri("getDests", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }

    fn get_bus_stops(
        &self,
        operator: &models::Operator,
    ) -> Box<Future<Item = models::BusStops, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting bus stops";
            "operator" => ?operator,
        );
        let uri_params = format!("operatorId={}", operator.to_string());
        let uri = match self.get_uri("getBusStops", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }
}
