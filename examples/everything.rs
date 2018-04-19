extern crate my_bus_tracker;

extern crate chrono;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate tokio_core;

use chrono::prelude::*;
use std::env;
use slog::Drain;
use slog::Logger;
use tokio_core::reactor::Core;

use my_bus_tracker::models;
use my_bus_tracker::TopologicalServices;
use my_bus_tracker::DisruptionsServices;
use my_bus_tracker::BusTimesService;

fn main() {
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let logger = Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!());

    info!(logger, "Launching Bus Notifier");

    let api_key = env::var("BUSNOTIFIER_MYBUSTRACKER_APIKEY")
        .expect("Missing API Key (BUSNOTIFIER_MYBUSTRACKER_APIKEY");

    let mut core = Core::new().expect("Couldn't get tokio core");
    let handle = core.handle();

    let bus_tracker = my_bus_tracker::MyBusTracker::new(&logger, &api_key, &handle).unwrap();

    let topo_id_future = bus_tracker.get_topo_id(&models::Operator::AllOperators);
    let topo_id = core.run(topo_id_future).expect("Error running function");
    println!("{:?}", topo_id);

    let services_future = bus_tracker.get_services(&models::Operator::AllOperators);
    let services: models::Services = core.run(services_future).expect("Error running function");
    println!("{:?}", services);

    let (some_service_ref, some_service_operator) = match services.services.get(0) {
        Some(service) => (service.reference.as_str(), &service.operator_id),
        None => panic!("No services found"),
    };
    let service_points_future =
        bus_tracker.get_service_points(some_service_ref, some_service_operator);
    let service_points = core.run(service_points_future)
        .expect("Error running function");
    println!("{:?}", service_points);

    let destinations_future = bus_tracker.get_destinations(&models::Operator::AllOperators);
    let destinations = core.run(destinations_future)
        .expect("Error running function");
    println!("{:?}", destinations);

    let bus_stops_future = bus_tracker.get_bus_stops(&models::Operator::AllOperators);
    let bus_stops: models::BusStops = core.run(bus_stops_future).expect("Error running function");
    println!("{:?}", bus_stops);

    let disruptions_future = bus_tracker.get_disruptions(&None, &models::Operator::AllOperators);
    let disruptions = core.run(disruptions_future)
        .expect("Error running function");
    println!("{:?}", disruptions);

    let diversions_future =
        bus_tracker.get_diversions(&None, &None, &models::Operator::AllOperators);
    let diversions: models::Diversions =
        core.run(diversions_future).expect("Error running function");
    println!("{:?}", diversions);

    let some_diversion_id = match diversions.diversions.get(0) {
        Some(diversion) => &diversion.diversion_id,
        None => panic!("No diversions found"),
    };

    let diversion_points_future =
        bus_tracker.get_diversion_points(some_diversion_id, &models::Operator::AllOperators);
    let diversion_points = core.run(diversion_points_future)
        .expect("Error running function");
    println!("{:?}", diversion_points);

    let stop_id = bus_stops.bus_stops[0].stop_id.clone();
    let service_id = bus_stops.bus_stops[0].services[0].clone();
    let destination_id = services
        .services
        .iter()
        .find(|service| service.reference == service_id)
        .expect("Non-existent service referenced")
        .destinations[0]
        .to_owned();
    let timetable = models::Timetable {
        stop_id: stop_id.clone(),
        service_reference: service_id,
        destination_reference: destination_id,
        operator_id: models::Operator::AllOperators,
    };
    let timetables = vec![timetable];
    let bus_times_future = bus_tracker.get_bus_times(&timetables, 1, &None, &None);
    let bus_times: models::BusTimes = core.run(bus_times_future).expect("Error running function");
    println!("{:?}", bus_times);

    let journey_times_future = bus_tracker.get_journey_times(
        &Some(&stop_id),
        &models::JourneyIdentifier::JourneyId(bus_times.bus_times[0].times[0].journey_id.clone()),
        &models::Operator::AllOperators,
        &Utc::today(),
        &models::JourneyTimeMode::All,
    );
    let journey_times = core.run(journey_times_future)
        .expect("Error running function");
    println!("{:?}", journey_times);
}
