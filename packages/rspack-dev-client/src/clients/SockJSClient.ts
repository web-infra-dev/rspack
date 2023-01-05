export * from "webpack-dev-server/client/clients/SockJSClient";

// TODO: hack providerPlugin
// @ts-ignored
if (typeof __webpack_require__ !== "undefined") {
	var id = "/ws-client";
	// @ts-ignored
	__webpack_require__.m[id] =
		// @ts-ignored
		__webpack_require__.m[id] ||
		function (module) {
			module.exports = require("webpack-dev-server/client/clients/SockJSClient");
		};
}
