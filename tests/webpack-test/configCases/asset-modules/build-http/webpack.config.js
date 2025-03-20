const path = require("path");

/** @type {import("../../../../").Configuration} */
module.exports = {
	entry: "./index.js",
	mode: "development",
	experiments: {
		buildHttp: {
			httpClient: {
				get: (url, headers) => {
					return fetch(url, { headers }).then(async res => {
						// Create a proper response object with status, headers, and body
						const body = res.text();

						// Convert headers to simple object
						const responseHeaders = {};
						res.headers.forEach((value, key) => {
							responseHeaders[key] = value;
						});

						return {
							status: res.status,
							headers: responseHeaders,
							body
						};
					});
				}
			},
			allowedUris: [() => true],
			lockfileLocation: path.resolve(__dirname, "./lock-files/lock.json"),
			cacheLocation: path.resolve(__dirname, "./lock-files/test")
		}
	}
};
