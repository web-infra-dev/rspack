import { createWebSocketClient } from "./ws";

export interface Handler {
	// TODO: remove data after jsonp
	ok(data: any): void;
	close(): void;
	"static-changed"(): void;
}

let retries = 0;
let maxRetries = 10;

const socket = function initSocket(
	url: string,
	handlers: Handler,
	reconnect?: number
) {
	let client = createWebSocketClient(url);
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

			console.info("Trying to reconnect...");

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
