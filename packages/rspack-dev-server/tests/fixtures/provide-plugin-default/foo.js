"use strict";

// 'npm run prepare' must be run for this to work during testing
const WebsocketClient =
	require("webpack-dev-server/client/clients/WebSocketClient").default;

window.expectedClient = WebsocketClient;
// eslint-disable-next-line camelcase, no-undef
window.injectedClient = __webpack_dev_server_client__.default;
