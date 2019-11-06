var map = L.map('map', {
	maxBounds: [
		[47.3, 5.9], // Southwest coordinates
		[54.9, 16.9512215] // Northeast coordinates
	],
}).setView([51.1657, 10.4515], 6);
L.tileLayer('https://api.tiles.mapbox.com/v4/{id}/{z}/{x}/{y}.png?access_token={accessToken}', {
	attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors, <a href="https://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>, Imagery © <a href="https://www.mapbox.com/">Mapbox</a>',
	maxZoom: 18,
	minZoom: 6,
	id: 'mapbox.streets',
	accessToken: 'pk.eyJ1IjoibWFwYm94IiwiYSI6ImNpejY4NXVycTA2emYycXBndHRqcmZ3N3gifQ.rJcFIG214AriISLbB6B5aw'
}).addTo(map);

map.on('click', onMapClick);

let url = "http://localhost:8080/";

let startPoint;
let startMarker;
let endPoint;
let endMarker;
let tmpMarker;
var last_path;
let xhr = new XMLHttpRequest();

function onMapClick(e) {
	if (tmpMarker) {
		map.removeLayer(tmpMarker);
	}
	tmpMarker = L.marker(e.latlng).addTo(map);
	tmpMarker.setLatLng(e.latlng);
	tmpMarker.bindPopup("<button class='set-point set-start' onclick='setStart()''>Set Start</button><button class='set-point set-end' onclick='setEnd()''>Set End</button>").openPopup();
}

function setStart() {
	let coords = tmpMarker.getLatLng();
	let lat = Math.round(coords.lat * 1000) / 1000;
	let lng = Math.round(coords.lng * 1000) / 1000;
	document.getElementById("start-text").innerHTML = "latitude: " + lat.toString() + "<br> longitude: " + lng.toString();
	if (startMarker) {
		map.removeLayer(startMarker);
	}
	startPoint = tmpMarker.getLatLng();
	startMarker = L.marker(coords, {
		icon: greenIcon
	}).addTo(map);
	map.removeLayer(tmpMarker);
}

function setEnd() {
	let coords = tmpMarker.getLatLng();
	let lat = Math.round(coords.lat * 1000) / 1000;
	let lng = Math.round(coords.lng * 1000) / 1000;
	document.getElementById("end-text").innerHTML = "latitude: " + lat.toString() + "<br> longitude: " + lng.toString();
	if (endMarker) {
		map.removeLayer(endMarker);
	}
	endPoint = tmpMarker.getLatLng();
	endMarker = L.marker(coords, {
		icon: redIcon
	}).addTo(map);
	map.removeLayer(tmpMarker);
}

function query() {
	hide_invalid_request();
	hide_no_path_found();
	hide_select_start_and_end();

	if (typeof last_path === 'object') {
		map.removeLayer(last_path);
	}
	
	if (typeof startPoint === 'undefined' || typeof endPoint === 'undefined') {
		show_select_start_and_end();
		return;
	}

	var xhr = new XMLHttpRequest();
	xhr.open("POST", url + "dijkstra", true);
	xhr.setRequestHeader("Content-type", "application/json;charset=UTF-8");

	xhr.onreadystatechange = function() {
		if (xhr.readyState === 4 && xhr.status === 200) {
			var json = JSON.parse(xhr.responseText);
			if (json.path != "") {
				printPath(json.path);
				show_result(json.cost);
			} else {
				show_no_path_found();
			}
		} else if (xhr.readyState === 4) {
			show_invalid_request();
		}
	};

	var travel_type = document.getElementById("travel-type").value == "car";
	var optimization = document.getElementById("optimization").value == "distance";
	var body = {
		"start": {
			"latitude": startPoint.lat,
			"longitude": startPoint.lng
		},
		"end": {
			"latitude": endPoint.lat,
			"longitude": endPoint.lng
		},
		"use_car": travel_type,
		"by_distance": optimization,
	};
	var data = JSON.stringify(body);
	// console.log("request: " + data);
	xhr.send(data);
}


function printPath(path) {
	// create [lat, lng] array for leaflet map
	let points = path.map(function(node) {
		return [node.latitude, node.longitude]
	});
	last_path = L.polyline(points);
	map.addLayer(last_path);
}


function show_invalid_request() {
	document.getElementById("invalid-request").style.display = "block";
}
function hide_invalid_request() {
	var x = document.getElementById("invalid-request");
	if (x.style.display === "block") {
		x.style.display = "none";
	}
}

function show_no_path_found() {
	document.getElementById("no-path-found").style.display = "block";
}
function hide_no_path_found() {
	var x = document.getElementById("no-path-found");
	if (x.style.display === "block") {
		x.style.display = "none";
	}
}

function show_select_start_and_end() {
	document.getElementById("select-start-and-end").style.display = "block";
}
function hide_select_start_and_end() {
	var x = document.getElementById("select-start-and-end");
	if (x.style.display === "block") {
		x.style.display = "none";
	}
}

function show_result(costs) {
	var tmp = document.getElementById("result")
	tmp.innerHTML = costs;
	tmp.style.display = "block";
}
function hide_result() {
	var x = document.getElementById("result");
	if (x.style.display === "block") {
		x.style.display = "none";
	}
}

var greenIcon = new L.Icon({
	iconUrl: 'img/marker-green.png',
	shadowUrl: 'img/marker-shadow.png',
	iconSize: [25, 41],
	iconAnchor: [12, 41],
	popupAnchor: [1, -34],
	shadowSize: [41, 41]
});
var redIcon = new L.Icon({
	iconUrl: 'img/marker-red.png',
	shadowUrl: 'img/marker-shadow.png',
	iconSize: [25, 41],
	iconAnchor: [12, 41],
	popupAnchor: [1, -34],
	shadowSize: [41, 41]
});