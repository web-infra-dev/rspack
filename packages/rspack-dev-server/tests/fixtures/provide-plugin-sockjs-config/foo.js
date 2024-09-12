"use strict";

// 'npm run prepare' must be run for this to work during testing
const SockJSClient =
	require("webpack-dev-server/client/clients/SockJSClient").default;

window.expectedClient = SockJSClient;
// eslint-disable-next-line camelcase, no-undef
window.injectedClient = __webpack_dev_server_client__.default;
