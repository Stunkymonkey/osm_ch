var map = L.map('map', {
	maxBounds: [
	    [47.3, 5.9], // Southwest coordinates
	    [54.9, 16.9512215]  // Northeast coordinates
	],}).setView([51.1657, 10.4515], 6);
L.tileLayer('https://api.tiles.mapbox.com/v4/{id}/{z}/{x}/{y}.png?access_token={accessToken}', {
	attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors, <a href="https://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>, Imagery © <a href="https://www.mapbox.com/">Mapbox</a>',
	maxZoom: 18,
	minZoom: 6,
	id: 'mapbox.streets',
	accessToken: 'pk.eyJ1IjoibWFwYm94IiwiYSI6ImNpejY4NXVycTA2emYycXBndHRqcmZ3N3gifQ.rJcFIG214AriISLbB6B5aw'
}).addTo(map);

let startPoint;
let startMarker;
let endPoint;
let endMarker;
let tmpMarker;
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
	if (startMarker) {
		map.removeLayer(startMarker);
	}
	startPoint = tmpMarker.getLatLng();
	startMarker = L.marker(tmpMarker.getLatLng(), {icon: greenIcon}).addTo(map);
	map.removeLayer(tmpMarker);
}

function setEnd() {
	if (endMarker) {
		map.removeLayer(endMarker);
	}
	endPoint = tmpMarker.getLatLng();
	endMarker = L.marker(tmpMarker.getLatLng(), {icon: redIcon}).addTo(map);
	map.removeLayer(tmpMarker);
}

function query() {
	document.getElementById("invalid-request").style.display = "none";
	xhr.open("POST", 'http://localhost:8080/dijkstra', true);
	xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
	xhr.send(
		JSON.stringify({
			"start":{
				"latitude": startPoint.lat,
				"longitude": startPoint.lng
			},
			"end":{
				"latitude": endPoint.lat,
				"longitude": endPoint.lng
			},
			"use_car" : true,
			"by_distance" : true,
		})
	);
	show_invalid_request();
}

map.on('click', onMapClick);

function hide_invalid_request() {
	var x = document.getElementById("invalid-request");
	if (x.style.display === "block") {
		x.style.display = "none";
	}
}

function show_invalid_request() {
	document.getElementById("invalid-request").style.display = "block";
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
