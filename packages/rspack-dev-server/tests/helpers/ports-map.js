"use strict";

// important: new port mappings must be added to the bottom of this list
const listOfTests = {
	// CLI tests
	"cli-basic": 1,
	"cli-port-option": 1,
	// e2e tests
	bundle: 1,
	"sockjs-client": 1,
	"web-socket-client": 1,
	"hot-and-live-reload": 1,
	logging: 1,
	overlay: 1,
	progress: 1,
	"server-and-client-transport": 1,
	"web-socket-server-url": 2,
	// integration tests
	"module-federation": 1,
	"multi-compiler": 1,
	// unit tests
	bonjour: 1,
	"client-option": 1,
	"compress-option": 1,
	"headers-option": 1,
	"history-api-fallback-option": 1,
	"host-option": 1,
	"hot-option": 1,
	"http2-option": 1,
	"https-option": 1,
	"mime-types-option": 1,
	"magic-html-option": 1,
	"on-after-setup-middleware-option": 1,
	"on-before-setup-middleware-option": 1,
	"on-listening-option": 1,
	"open-option": 1,
	"port-option": 1,
	"proxy-option": 4,
	server: 1,
	"setup-exit-signals-option": 1,
	"static-directory-option": 1,
	"static-public-path-option": 1,
	"stats-option": 1,
	"watch-files-option": 1,
	"web-socket-server-option": 1,
	"sockjs-server": 1,
	"web-socket-server": 1,
	routes: 1,
	"web-socket-communication": 1,
	ipc: 1,
	stats: 1,
	"cli-allowed-hosts": 1,
	"cli-bonjour": 1,
	"cli-client": 1,
	"cli-colors": 1,
	"cli-compress": 1,
	"cli-history-api-fallback": 1,
	"cli-host": 1,
	"cli-hot": 1,
	"cli-http2": 1,
	"cli-https": 1,
	"cli-live-reload": 1,
	"cli-magic-html": 1,
	"cli-open": 1,
	"cli-static": 1,
	"cli-watch-files": 1,
	"cli-web-socket-server": 1,
	target: 1,
	entry: 1,
	"allowed-hosts": 2,
	host: 1,
	api: 1,
	"lazy-compilation": 1,
	"range-header": 1,
	port: 1,
	"web-socket-server-test": 1,
	"client-reconnect-option": 1,
	"cli-server": 1,
	"server-option": 1,
	"normalize-option": 1,
	"setup-middlewares-option": 1,
	"options-request-response": 2,
	app: 1
};

let startPort = 8089;

const ports = {};

Object.keys(listOfTests).forEach(key => {
	const value = listOfTests[key];

	ports[key] =
		value === 1
			? (startPort += 1)
			: [...new Array(value)].map(() => (startPort += 1));
});

const busy = {};

module.exports = new Proxy(ports, {
	get(target, name) {
		if (!target[name]) {
			throw new Error(
				`Requested "${name}" port(s) for tests not found, please update "test/ports-map.js".`
			);
		}

		if (busy[name]) {
			throw new Error(
				`The "${name}" port is already in use in another test, please add a new one.`
			);
		}

		busy[name] = true;

		return target[name];
	}
});
