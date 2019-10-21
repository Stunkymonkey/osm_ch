var map = L.map('map').setView([51.1657, 10.4515], 6);
L.tileLayer('https://api.tiles.mapbox.com/v4/{id}/{z}/{x}/{y}.png?access_token={accessToken}', {
	attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors, <a href="https://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>, Imagery © <a href="https://www.mapbox.com/">Mapbox</a>',
	maxZoom: 18,
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
		xhr.send(JSON.stringify({
			"start":
				{	"latitude": startPoint.lat,
					"longitude": startPoint.lng
				},
			"end":
				{	"latitude": endPoint.lat,
					"longitude": endPoint.lng
			}
		}));
	}
}

map.on('click', onMapClick);
