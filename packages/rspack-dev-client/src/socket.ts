/**
 * The following code is modified based on
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/client-src/socket.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/LICENSE
 */
import WebSocketClient from "webpack-dev-server/client/clients/WebSocketClient";
import { log } from "webpack-dev-server/client/utils/log";

// @ts-ignore
const __webpack_dev_server_client__ = __webpack_modules__.$WsClient$;
const Client =
	typeof __webpack_dev_server_client__ !== "undefined"
		? typeof __webpack_dev_server_client__.default !== "undefined"
			? __webpack_dev_server_client__.default
			: __webpack_dev_server_client__
		: WebSocketClient;

let retries = 0;
let maxRetries = 10;
export let client = null;

const socket = function initSocket(
	url: string,
	handlers: any,
	reconnect?: number
) {
	let client = new Client(url);
	client.onOpen(() => {
		retries = 0;

		if (typeof reconnect !== "undefined") {
			maxRetries = reconnect;
		}
	});

	client.onClose(() => {
		if (retries === 0) {
			handlers.close();
		}

		client = null;

		if (retries < maxRetries) {
			const retryInMs = 1000 * Math.pow(2, retries) + Math.random() * 100;

			retries += 1;

			log.info("Trying to reconnect...");

			setTimeout(() => {
				socket(url, handlers, reconnect);
			}, retryInMs);
		}
	});

	client.onMessage((data: any) => {
		const message = JSON.parse(data);
		if (handlers[message.type]) {
			handlers[message.type](message.data, message.params);
		}
	});
};

export default socket;
