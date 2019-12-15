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
// TODO check if more types can be added
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
        "path" | "walk" | "footway" => 4,
        _ => 50,
    };
}

/// get what kind of street it is:
/* infos from https://wiki.openstreetmap.org/wiki/Key:highway
0 = car only
1 = car and bicycle
2 = bicycle
3 = bicycle and pedestrian
4 = pedestrian
5 = all
100 = skip
*/
pub fn get_street_type(s: &str, has_sidewalk: bool) -> usize {
    let mut result = match s {
        "motorway" | "motorway_link" => 0,
        "trunk" | "trunk_link" => 0,
        "raceway" | "services" | "rest_area" => 0,
        "primary" | "primary_link" => 1,
        "secondary" | "secondary_link" => 1,
        "tertiary" | "tertiary_link" => 1,
        "cycleway" => 2,
        "trail" | "track" | "path" => 3,
        "elevator" | "platform" | "corridor" => 4,
        "bus_stop" | "bridleway" | "steps" | "pedestrian" | "footway" => 4,
        "unclassified" => 5,
        "residential" | "living_street" => 5,
        "service" | "road" => 5,
        "razed" | "abandoned" | "disused" | "construction" | "proposed" => 100,
        _ => 5,
    };
    if has_sidewalk {
        result = match result {
            1 => 5,
            2 => 3,
            3 => 5,
            _ => result,
        }
    }
    return result;
}
