use super::*;

// parse max-speed to valid weight (fallback is highway-tag)
pub fn parse_speed(max_speed: &str, highway: &str) -> usize {
    match max_speed.parse::<usize>() {
        Ok(ok) => return ok,
        Err(_e) => match resolve_max_speed(max_speed) {
            Ok(ok) => return ok,
            Err(_e) => {
                return aproximate_speed_limit(highway);
            }
        },
    }
}

/// resolves the int value from a dirty string that can't be resolved by default parsing
fn resolve_max_speed(s: &str) -> Result<usize, &str> {
    return match s {
        "DE:motorway" => Ok(120),
        "DE:rural" | "AT:rural" => Ok(100),
        "DE:urban" | "AT:urban" | "CZ:urban" => Ok(50),
        "maxspeed=50" => Ok(50),
        "50;" | "50b" => Ok(50),
        "DE:living_street" => Ok(30),
        "30 kph" => Ok(30),
        "zone:maxspeed=de:30" => Ok(30),
        "DE:zone:30" => Ok(30),
        "DE:zone30" => Ok(30),
        "30 mph" => Ok(30),
        "20:forward" => Ok(20),
        "10 mph" => Ok(10),
        "5 mph" => Ok(7),
        "DE:walk" | "walk" | "Schrittgeschwindigkeit" => Ok(7),
        _ => Err("none"),
    };
}

/// approximates the speed limit based on given highway type
// infos from https://wiki.openstreetmap.org/wiki/Key:highway
fn aproximate_speed_limit(s: &str) -> usize {
    return match s {
        "motorway" => 120,
        "motorway_link" => 60,
        "trunk" => 100,
        "trunk_link" => 50,
        "primary" => 60,
        "primary_link" => 50,
        "secondary" | "secondary_link" => 50,
        "tertiary" | "tertiary_link" => 50,
        "unclassified" => 40,
        "residential" => 30,
        "track" | "service" => 10,
        "living_street" => 7,
        "path" | "walk" | "pedestrian" | "footway" => 4,
        _ => 50,
    };
}

/// get what kind of street it is:
// infos from https://wiki.openstreetmap.org/wiki/Key:highway
pub fn get_street_type(s: &str, has_sidewalk: bool) -> TravelType {
    let mut result = match s {
        "motorway" | "motorway_link" => TravelType::Car,
        "trunk" | "trunk_link" => TravelType::Car,
        "raceway" | "services" | "rest_area" => TravelType::Car,
        "primary" | "primary_link" => TravelType::CarBicycle,
        "secondary" | "secondary_link" => TravelType::CarBicycle,
        "tertiary" | "tertiary_link" => TravelType::CarBicycle,
        "cycleway" => TravelType::Bicycle,
        "trail" | "track" | "path" => TravelType::BicyclePedestrian,
        "elevator" | "platform" | "corridor" => TravelType::Pedestrian,
        "bus_stop" | "bridleway" | "steps" | "pedestrian" | "footway" => TravelType::Pedestrian,
        "unclassified" => TravelType::All,
        "residential" | "living_street" => TravelType::All,
        "service" | "road" => TravelType::All,
        "razed" | "abandoned" | "disused" | "construction" | "proposed" => TravelType::Undefined,
        _ => TravelType::All,
    };
    if has_sidewalk {
        result = match result {
            TravelType::CarBicycle => TravelType::All,
            TravelType::Bicycle => TravelType::BicyclePedestrian,
            _ => result,
        }
    }
    return result;
}

/// get directions from on_way
// info from: https://wiki.openstreetmap.org/wiki/Forward_%26_backward,_left_%26_right#Identifying_the_direction_of_a_way
pub fn parse_one_way(s: &str) -> (bool, bool) {
    return match s {
        "yes" => (true, false),
        "-1" => (true, true),
        _ => (false, false),
    };
}
