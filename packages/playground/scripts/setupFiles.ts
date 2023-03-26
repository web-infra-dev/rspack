import path from "path";
import { RspackDevServer } from "@rspack/dev-server";
import { createCompiler } from "@rspack/core";
import { WebSocket } from "ws";

let server: RspackDevServer;

beforeAll(async () => {
	const { page } = global;
	if (!page) {
		throw Error("make sure connect before test");
	}
	const { testPath } = expect.getState();
	if (!testPath) {
		return;
	}
	const caseName = testPath
		.replace(/\\/g, "/")
		.match(/playground\/fixtures\/([\w-]+)\//)?.[1];
	if (!caseName) {
		return;
	}
	const testDir = path.resolve(__dirname, "../temp", caseName);
	const configPath = path.resolve(testDir, "webpack.config.js");
	const configFile = require(configPath);
	const compiler = createCompiler(configFile);
	server = new RspackDevServer(compiler.options.devServer ?? {}, compiler);
	await server.start();
	await waitingForBuild(server.options.port);
	const url = `http://localhost:${server.options.port}`;
	await page.goto(url);
});

afterAll(async () => {
	await global.page?.close();
	await server.stop();
});

async function waitingForBuild(port: number | string) {
	await new Promise(resolve => {
		const ws = new WebSocket(`ws://127.0.0.1:${port}/ws`, {
			headers: {
				host: `127.0.0.1:${port}`,
				origin: `http://127.0.0.1:${port}`
			}
		});

		let opened = false;
		let received = false;
		let errored = false;

		ws.on("error", error => {
			// @ts-ignore
			if (/404/.test(error)) {
				errored = true;
			} else {
				errored = true;
			}

			ws.close();
		});

		ws.on("open", () => {
			opened = true;
		});

		ws.on("message", data => {
			// @ts-ignore
			const message = JSON.parse(data);

			if (message.type === "ok") {
				received = true;

				ws.close();
			}
		});

		ws.on("close", () => {
			if (opened && received && !errored) {
				resolve(undefined);
			} else if (errored) {
				resolve(undefined);
			}
		});
	});
}
