//! Bus Times Web Service
//!
//! For full documentation, see Section IV.4 of the My Bus Tracker API Guide (Version F)

use hyper::{Method, Request};
use super::{models, MyBusTracker, MyBusTrackerError};
use futures::{self, Future};
use chrono::{Date, Duration, NaiveTime, Utc};

/// Bus Times Web Service
///
/// To use methods from the Bus Times Web Service, bring this trait into scope
/// alongside your `MyBusTracker` instance.
pub trait BusTimesService {
    /// Get a list of timetables
    ///
    /// You may request:
    ///   - between 1 and 5 `timetables`, inclusive;
    ///   - optionally, between 1 and 10 `departure_count`s, inclusive - the default is 2;
    ///   - optionally, a date, up to three-days in the future - the default is today;
    ///   - optionally, a time - the default is now.
    fn get_bus_times(
        &self,
        timetables: &[models::Timetable],
        departure_count: u8,
        departure_day: &Option<&Date<Utc>>,
        departure_time: &Option<&NaiveTime>,
    ) -> Box<Future<Item = models::BusTimes, Error = MyBusTrackerError>>;

    /// Get a list of bus arrival times
    ///
    /// You may request details on:
    ///   - a journey identifier, either a Journey ID or a Bus Fleet Number
    ///   - optionally, a specific stop - if the journey identifier is a Journey ID,
    ///     this is not optional
    fn get_journey_times(
        &self,
        stop_id: &Option<&str>,
        journey_id: &models::JourneyIdentifier,
        operator: &models::Operator,
        day: &Date<Utc>,
        mode: &models::JourneyTimeMode,
    ) -> Box<Future<Item = models::JourneyTimes, Error = MyBusTrackerError>>;
}

impl BusTimesService for MyBusTracker {
    fn get_bus_times(
        &self,
        timetables: &[models::Timetable],
        departure_count: u8,
        departure_day: &Option<&Date<Utc>>,
        departure_time: &Option<&NaiveTime>,
    ) -> Box<Future<Item = models::BusTimes, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting bus times";
            "timetables" => ?timetables,
            "departures" => departure_count,
            "departure_time" => ?departure_time,
            "departure_day" => ?departure_day,
        );
        if timetables.len() > 5 {
            return Box::new(futures::failed(MyBusTrackerError::TooManyTimetables));
        }

        if departure_count > 10 {
            return Box::new(futures::failed(MyBusTrackerError::TooManyDepartures));
        }

        let day_difference: Duration = match *departure_day {
            Some(departure_day) => departure_day.signed_duration_since(Utc::today()),
            None => Duration::days(0),
        };

        if day_difference > Duration::days(3) || day_difference < Duration::days(0) {
            return Box::new(futures::failed(MyBusTrackerError::DateOutOfBounds));
        }

        let departure_time_string = match *departure_time {
            Some(time) => format!("&time={}", time.format("%H:%M")),
            None => String::new(),
        };

        let time_requests = timetables
            .iter()
            .enumerate()
            .map(|(i, item)| {
                format!(
                    "stopId{0}={1}&refService{0}={2}&refDest{0}={3}",
                    i + 1,
                    item.stop_id,
                    item.service_reference,
                    item.destination_reference
                )
            })
            .collect::<Vec<String>>()
            .join("&");

        let uri_params = format!(
            "{}&nb={}&day={}{}",
            time_requests,
            departure_count,
            day_difference.num_days(),
            departure_time_string
        );
        let uri = match self.get_uri("getBusTimes", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }

    fn get_journey_times(
        &self,
        stop_id: &Option<&str>,
        journey_id: &models::JourneyIdentifier,
        operator: &models::Operator,
        day: &Date<Utc>,
        mode: &models::JourneyTimeMode,
    ) -> Box<Future<Item = models::JourneyTimes, Error = MyBusTrackerError>> {
        debug!(
            self.logger,
            "Getting journey times";
            "journey_id" => ?journey_id,
            "stop_id" => stop_id,
            "operator" => ?operator,
            "day" => ?day,
            "mode" => ?mode,
        );

        let stop_id_string = match *stop_id {
            Some(stop) => format!("stopId={}&", stop),
            None => String::new(),
        };

        let journey_id_string = match *journey_id {
            models::JourneyIdentifier::JourneyId(ref journey) => format!("journeyId={}&", journey),
            models::JourneyIdentifier::BusId(ref bus) => format!("busId={}&", bus),
        };

        let day_difference: Duration = day.signed_duration_since(Utc::today());
        if day_difference > Duration::days(3) || day_difference < Duration::days(0) {
            return Box::new(futures::failed(MyBusTrackerError::DateOutOfBounds));
        }

        let uri_params = format!(
            "{}{}operator={}&day={}&mode={}",
            stop_id_string,
            journey_id_string,
            operator,
            day_difference.num_days(),
            mode
        );

        let uri = match self.get_uri("getJourneyTimes", Some(&uri_params)) {
            Ok(uri) => uri,
            Err(uri_error) => return Box::new(futures::failed(uri_error)),
        };

        let request = Request::new(Method::Get, uri);

        self.make_request(request)
    }
}
