# OSM-Dijkstra
by Felix Bühler and Simon Matejetz

This project was part of "Lab Course: Algorithms for OSM Data".

This repository consists of two programms:

## pre

this will parse the `*.osm.pbf` file into a `*.osm.pbf.fmi` file, which is needed for the `web`-program

### dependecies

- `osmpbfreader` = parsing the pbf file
- `serde` = serialization
- `bincode` = exporting serialization

### Compilation
`cargo build --release`

### Usage
`cargo run --release ./germany-latest.osm.pbf`

## web

is the webserver which provides the interface. it needs the `*.osm.pbf.fmi`-file from the `pre`-programm.

### dependecies

- `actix-files` = serving static files
- `actix-web` = webserver
- `serde` = serialization
- `bincode` = exporting serialization
- `serde_json` = parsing json

### Compilation

`cargo build --release`

### Usage

`cargo run --release ./germany-latest.osm.pbf.fmi`