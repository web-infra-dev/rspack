"use strict";

// require("./helpers/warmup-webpack");

const { createFsFromVolume, Volume } = require("memfs");

const compile = options => {
	return new Promise((resolve, reject) => {
		const webpack = require("@rspack/core").rspack;
		const compiler = webpack(options);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) {
				reject(err);
			} else {
				resolve(stats);
			}
		});
	});
};

describe("Stats", () => {
	// it("should print env string in stats", async () => {
	// 	const stats = await compile({
	// 		context: __dirname,
	// 		entry: "./fixtures/a"
	// 	});
	// 	expect(
	// 		stats.toString({
	// 			all: false,
	// 			env: true,
	// 			_env: "production"
	// 		})
	// 	).toBe('Environment (--env): "production"');
	// 	expect(
	// 		stats.toString({
	// 			all: false,
	// 			env: true,
	// 			_env: {
	// 				prod: ["foo", "bar"],
	// 				baz: true
	// 			}
	// 		})
	// 	).toBe(
	// 		"Environment (--env): {\n" +
	// 			'  "prod": [\n' +
	// 			'    "foo",\n' +
	// 			'    "bar"\n' +
	// 			"  ],\n" +
	// 			'  "baz": true\n' +
	// 			"}"
	// 	);
	// });
	it("should omit all properties with all false", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/a"
		});
		expect(
			stats.toJson({
				all: false
			})
		).toEqual({});
	});
	describe("chunkGroups", () => {
		it("should be empty when there is no additional chunks", async () => {
			const stats = await compile({
				context: __dirname,
				entry: {
					entryA: "./fixtures/a",
					entryB: "./fixtures/b"
				}
			});
			expect(
				stats.toJson({
					all: false,
					errorsCount: true,
					chunkGroups: true
				})
			).toMatchInlineSnapshot(`
			{
			  "errorsCount": 1,
			}
		`);
		});
		it("should contain additional chunks", async () => {
			const stats = await compile({
				context: __dirname,
				entry: {
					entryA: "./fixtures/a",
					entryB: "./fixtures/chunk-b"
				}
			});
			expect(
				stats.toJson({
					all: false,
					errorsCount: true,
					chunkGroups: true
				})
			).toMatchInlineSnapshot(`
			{
			  "errorsCount": 1,
			}
		`);
		});
		it("should contain assets", async () => {
			const stats = await compile({
				context: __dirname,
				entry: {
					entryA: "./fixtures/a",
					entryB: "./fixtures/chunk-b"
				}
			});
			expect(
				stats.toJson({
					all: false,
					errorsCount: true,
					assets: true
				})
			).toMatchInlineSnapshot(`
			{
			  "assets": [
			    {
			      "chunkNames": [
			        "entryB",
			      ],
			      "chunks": [
			        "entryB",
			      ],
			      "emitted": true,
			      "info": {
			        "development": false,
			        "hotModuleReplacement": false,
			      },
			      "name": "entryB.js",
			      "size": 4772,
			      "type": "asset",
			    },
			    {
			      "chunkNames": [
			        "entryA",
			      ],
			      "chunks": [
			        "entryA",
			      ],
			      "emitted": true,
			      "info": {
			        "development": false,
			        "hotModuleReplacement": false,
			      },
			      "name": "entryA.js",
			      "size": 3598,
			      "type": "asset",
			    },
			    {
			      "chunkNames": [],
			      "chunks": [
			        "fixtures_b_js",
			      ],
			      "emitted": true,
			      "info": {
			        "development": false,
			        "hotModuleReplacement": false,
			      },
			      "name": "fixtures_b_js.js",
			      "size": 157,
			      "type": "asset",
			    },
			  ],
			  "assetsByChunkName": {
			    "entryA": [
			      "entryA.js",
			    ],
			    "entryB": [
			      "entryB.js",
			    ],
			  },
			  "errorsCount": 1,
			}
		`);
		});
	});
});
