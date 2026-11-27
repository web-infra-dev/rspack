Promise.all([
	import(/* webpackChunkName: "bundle" */ "./route.js"),
	import(/* webpackChunkName: "assets/bundle" */ "./other-route.js")
]).then(([route, otherRoute]) => console.log(route.default, otherRoute.default));
