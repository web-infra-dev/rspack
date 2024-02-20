/**
 * Copy from webpack-dev-server
 */

"use strict";

const SockJS = require("sockjs-client/dist/sockjs");

module.exports = class SockJSClient {
	constructor(url) {
		this.sock = new SockJS(
			url.replace(/^ws:/i, "http://").replace(/^wss:/i, "https://")
		);
	}

	onOpen(f) {
		this.sock.onopen = () => {
			console.log("open");
			f();
		};
	}

	onClose(f) {
		this.sock.onclose = () => {
			console.log("close");
			f();
		};
	}

	// call f with the message string as the first argument
	onMessage(f) {
		this.sock.onmessage = e => {
			const obj = JSON.parse(e.data);
			console.log(obj.type);
			f(e.data);
		};
	}
};
