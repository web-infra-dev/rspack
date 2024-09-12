"use strict";

const request = require("supertest");
const webpack = require("@rspack/core");
const { RspackDevServer: Server } = require("@rspack/dev-server");
const config = require("../fixtures/static-config/webpack.config");
const port = require("../helpers/ports-map")["range-header"];

describe("'Range' header", () => {
	let compiler;
	let server;

	beforeAll(async () => {
		compiler = webpack(config);

		server = new Server({ port }, compiler);

		await server.start();
	});

	afterAll(async () => {
		await server.stop();
	});

	it('should work with "Range" header using "GET" method', async () => {
		const response = await request(server.app).get("/main.js");

		expect(response.status).toBe(200);
		expect(response.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(response.headers["accept-ranges"]).toBe("bytes");

		const responseContent = response.text;
		const responseRange = await request(server.app)
			.get("/main.js")
			.set("Range", "bytes=0-499");

		expect(responseRange.status).toBe(206);
		expect(responseRange.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(responseRange.headers["content-length"]).toBe("500");
		expect(responseRange.headers["content-range"]).toMatch(/^bytes 0-499\//);
		expect(responseRange.text).toBe(responseContent.slice(0, 500));
		expect(responseRange.text.length).toBe(500);
	});

	it('should work with "Range" header using "HEAD" method', async () => {
		const response = await request(server.app).head("/main.js");

		expect(response.status).toBe(200);
		expect(response.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(response.headers["accept-ranges"]).toBe("bytes");

		const responseRange = await request(server.app)
			.head("/main.js")
			.set("Range", "bytes=0-499");

		expect(responseRange.status).toBe(206);
		expect(responseRange.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(responseRange.headers["content-length"]).toBe("500");
		expect(responseRange.headers["content-range"]).toMatch(/^bytes 0-499\//);
	});

	it('should work with unsatisfiable "Range" header using "GET" method', async () => {
		const response = await request(server.app).get("/main.js");

		expect(response.status).toBe(200);
		expect(response.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(response.headers["accept-ranges"]).toBe("bytes");

		const responseRange = await request(server.app)
			.get("/main.js")
			.set("Range", "bytes=99999999999-");

		expect(responseRange.status).toBe(416);
		expect(responseRange.headers["content-type"]).toBe(
			"text/html; charset=utf-8"
		);
		expect(responseRange.headers["content-range"]).toMatch(/^bytes \*\//);
	});

	it('should work with malformed "Range" header using "GET" method', async () => {
		const response = await request(server.app).get("/main.js");

		expect(response.status).toBe(200);
		expect(response.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(response.headers["accept-ranges"]).toBe("bytes");

		const responseContent = response.text;
		const responseRange = await request(server.app)
			.get("/main.js")
			.set("Range", "bytes");

		expect(responseRange.status).toBe(200);
		expect(responseRange.headers["content-type"]).toBe(
			"application/javascript; charset=utf-8"
		);
		expect(responseRange.text).toBe(responseContent);
		expect(responseRange.text.length).toBe(responseContent.length);
	});
});
