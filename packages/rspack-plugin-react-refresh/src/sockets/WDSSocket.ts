/**
 * The following code is modified based on
 * https://github.com/pmmmwh/react-refresh-webpack-plugin/blob/f1c8b9a44198449093ca95f85af5df97925e1cfc/sockets/WPSSocket.js
 *
 * MIT Licensed
 * Author Michael Mok
 * Copyright (c) 2019 Michael Mok
 * https://github.com/pmmmwh/react-refresh-webpack-plugin/blob/0b960573797bf38926937994c481e4fec9ed8aa6/LICENSE
 */
import getSocketUrlParts from "./utils/getSocketUrlParts";
import getUrlFromParts from "./utils/getUrlFromParts";
import getWDSMetadata from "./utils/getWDSMetadata";

declare global {
	var __webpack_dev_server_client__: any;
}

/**
 * Initializes a socket server for HMR for webpack-dev-server.
 * @param {function(*): void} messageHandler A handler to consume Webpack compilation messages.
 * @param {string} [resourceQuery] Webpack's `__resourceQuery` string.
 * @returns {void}
 */
export function init(
	messageHandler: (...args: any[]) => void,
	resourceQuery: string
) {
	if (typeof __webpack_dev_server_client__ !== "undefined") {
		let SocketClient = __webpack_dev_server_client__;
		if (typeof __webpack_dev_server_client__.default !== "undefined") {
			SocketClient = __webpack_dev_server_client__.default;
		}

		const wdsMeta = getWDSMetadata(SocketClient);
		const urlParts = getSocketUrlParts(resourceQuery, wdsMeta);

		const connection = new SocketClient(getUrlFromParts(urlParts, wdsMeta));

		// @ts-expect-error -- ignore
		connection.onMessage(function onSocketMessage(data) {
			const message = JSON.parse(data);
			messageHandler(message);
		});
	}
}
