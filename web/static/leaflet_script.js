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
let xhr = new XMLHttpRequest();


function onMapClick(e) {
	if (!startPoint || endPoint) {
		startPoint = e.latlng;
		endPoint = null;
		if(!startMarker) {
			startMarker = L.marker(e.latlng).addTo(map);
		}
		startMarker.setLatLng(e.latlng);
		startMarker.bindPopup("Start<br>" + e.latlng).openPopup();
	}
	else if(!endPoint) {
		endPoint = e.latlng;
		if(!endMarker) {
			endMarker = L.marker(e.latlng).addTo(map);
		}
		endMarker.setLatLng(e.latlng);
		endMarker.bindPopup("End<br>" + e.latlng).openPopup();
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
	}
}

map.on('click', onMapClick);

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

L.marker([52.12, 10.57], {icon: greenIcon}).addTo(map);
L.marker([50.68, 9.21], {icon: redIcon}).addTo(map);