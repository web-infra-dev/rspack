import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";

const hotDir = path.resolve(
	path.dirname(fileURLToPath(import.meta.url)),
	"../../../../packages/rspack/hot"
);

function loadScript(source, context) {
	vm.runInContext(source, context);
	return context.activate;
}

async function loadWebClient() {
	const source = (await readFile(path.join(hotDir, "lazy-compilation-web.js"), "utf8"))
		.replace("export const activate", "var activate")
		.replaceAll("import.meta.webpackHot", "true");
	const requests = [];
	class XMLHttpRequest {
		open() {}
		setRequestHeader() {}
		send(body) {
			requests.push({ body, xhr: this });
		}
		complete(status = 200) {
			this.readyState = 4;
			this.status = status;
			this.onreadystatechange();
		}
	}
	const context = vm.createContext({
		XMLHttpRequest,
		console,
		__resourceQuery: "?" + encodeURIComponent("http://127.0.0.1/_rspack/lazy/trigger")
	});
	const activate = loadScript(source, context);
	return { activate, requests };
}

async function loadNodeClient() {
	const source = (await readFile(path.join(hotDir, "lazy-compilation-node.js"), "utf8"))
		.replace("import { createRequire } from 'node:module';\n", "")
		.replace(
			"var require = createRequire(import.meta.url);",
			"var require = __mockRequire;"
		)
		.replace("export const activate", "var activate");
	const requests = [];
	function httpModule() {}
	httpModule.request = function (_url, _options, callback) {
		const request = {
			body: "",
			write(data) {
				this.body += data;
			},
			end() {
				requests.push(request);
			},
			on() {},
			complete(statusCode = 200) {
				callback({ statusCode, resume() {} });
			}
		};
		return request;
	};
	const context = vm.createContext({
		console,
		__mockRequire: () => httpModule,
		__resourceQuery: "?" + encodeURIComponent("http://127.0.0.1/_rspack/lazy/trigger")
	});
	const activate = loadScript(source, context);
	return { activate, requests };
}

function activatePair(activate) {
	const disposeA = activate({
		data: "lazy-compilation-proxy|a.js",
		active: false,
		module: { hot: {} },
		onError(error) {
			throw error;
		}
	});
	const disposeB = activate({
		data: "lazy-compilation-proxy|b.js",
		active: false,
		module: { hot: {} },
		onError(error) {
			throw error;
		}
	});
	return { disposeA, disposeB };
}

function complete(request) {
	if (request.xhr) request.xhr.complete();
	else request.complete();
}

async function assertDisposalDoesNotResend(load) {
	const { activate, requests } = await load();
	const { disposeA } = activatePair(activate);
	assert.equal(requests.length, 1);
	complete(requests[0]);
	assert.equal(requests.length, 2);
	assert.equal(
		requests[1].body,
		"lazy-compilation-proxy|a.js\nlazy-compilation-proxy|b.js"
	);
	disposeA();
	assert.equal(
		requests.length,
		2,
		"disposing one proxy must not post the ids that are still active"
	);
}

async function assertInFlightUpdateStillFlushes(load) {
	const { activate, requests } = await load();
	const disposeA = activate({
		data: "lazy-compilation-proxy|a.js",
		active: false,
		module: { hot: {} },
		onError(error) {
			throw error;
		}
	});
	assert.equal(requests.length, 1);
	activate({
		data: "lazy-compilation-proxy|b.js",
		active: false,
		module: { hot: {} },
		onError(error) {
			throw error;
		}
	});
	assert.equal(requests.length, 1);
	disposeA();
	const first = requests[0];
	if (first.xhr) first.xhr.complete();
	else first.complete();
	assert.equal(requests.length, 2);
	const secondBody = requests[1].body;
	assert.equal(secondBody, "lazy-compilation-proxy|b.js");
}

export async function run() {
	await assertDisposalDoesNotResend(loadWebClient);
	await assertDisposalDoesNotResend(loadNodeClient);
	await assertInFlightUpdateStillFlushes(loadWebClient);
	await assertInFlightUpdateStillFlushes(loadNodeClient);
}
