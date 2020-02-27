# OSM-Contraction-Hierarchies
by Felix Bühler

This project was part of "Lab Course: Algorithms for OSM Data".

This project implements "Contraction Hierarchies". Which is one of the best known speed-up techniques for shortest path calculations. As data source [OpenStreetMap](https://www.openstreetmap.org)-data is used.

![screenshot](./screenshot-osm_ch.png)

This repository consists of two programms:

## pre

This will parse the `*.osm.pbf` file into a `*.osm.pbf.fmi` file, which is needed for the `web`-program
Cropped OSM-data can be downloaded from [Geofabrik.de](https://download.geofabrik.de/index.html)

### dependecies

- `osmpbfreader` = parsing the pbf file
- `serde` = serialization
- `rayon` = parallelization
- `bincode` = exporting serialization

### Compilation
`cargo build --release -p osm_ch_pre`

### Usage
`cargo run --release -p osm_ch_pre ./germany-latest.osm.pbf`

## web

is the webserver which provides the web-interface. (it needs the `*.osm.pbf.fmi`-file from the `pre`-programm.)

### dependecies

- `actix-files` = serving static files
- `actix-rt` = running actix
- `actix-web` = webserver
- `bincode` = exporting serialization
- `env_logger` = logging the webserver
- `rayon` = parallelization
- `serde` = serialization
- `serde_json` = serialization json

### Compilation

`cargo build --release -p osm_ch_web`

### Usage

`cargo run --release -p osm_ch_web ./germany-latest.osm.pbf.`