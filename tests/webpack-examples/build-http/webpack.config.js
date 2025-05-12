module.exports = {
	// enable debug logging to see network requests!
	// stats: {
	// 	loggingDebug: /HttpUriPlugin/
	// },cd
	experiments: {

		buildHttp: {
			allowedUris: [
				"https://cdn.esm.sh/",
				"https://cdn.skypack.dev/",
				"https://jspm.dev/",
				/^https:\/\/unpkg\.com\/.+\?module$/
			]
		}
	}
};
