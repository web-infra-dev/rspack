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
			  "namedChunkGroups": {
			    "entryA": {
			      "assets": [
			        {
			          "name": "entryA.css",
			          "size": 0,
			        },
			        {
			          "name": "entryA.js",
			          "size": 215,
			        },
			      ],
			      "assetsSize": 215,
			      "chunks": [
			        "entryA",
			      ],
			      "name": "entryA",
			    },
			    "entryB": {
			      "assets": [
			        {
			          "name": "entryB.css",
			          "size": 0,
			        },
			        {
			          "name": "entryB.js",
			          "size": 227,
			        },
			      ],
			      "assetsSize": 227,
			      "chunks": [
			        "entryB",
			      ],
			      "name": "entryB",
			    },
			  },
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
			  "namedChunkGroups": {
			    "chunkB": {
			      "assets": [
			        {
			          "name": "chunkB.css",
			          "size": 0,
			        },
			        {
			          "name": "chunkB.js",
			          "size": 160,
			        },
			      ],
			      "assetsSize": 160,
			      "chunks": [
			        "chunkB",
			      ],
			      "name": "chunkB",
			    },
			    "entryA": {
			      "assets": [
			        {
			          "name": "entryA.css",
			          "size": 0,
			        },
			        {
			          "name": "entryA.js",
			          "size": 3608,
			        },
			      ],
			      "assetsSize": 3608,
			      "chunks": [
			        "entryA",
			      ],
			      "name": "entryA",
			    },
			    "entryB": {
			      "assets": [
			        {
			          "name": "entryB.css",
			          "size": 0,
			        },
			        {
			          "name": "entryB.js",
			          "size": 4747,
			        },
			      ],
			      "assetsSize": 4747,
			      "chunks": [
			        "entryB",
			      ],
			      "name": "entryB",
			    },
			  },
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
			      "size": 4747,
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
			      "size": 3608,
			      "type": "asset",
			    },
			    {
			      "chunkNames": [
			        "chunkB",
			      ],
			      "chunks": [
			        "chunkB",
			      ],
			      "emitted": true,
			      "info": {
			        "development": false,
			        "hotModuleReplacement": false,
			      },
			      "name": "chunkB.js",
			      "size": 160,
			      "type": "asset",
			    },
			    {
			      "chunkNames": [
			        "chunkB",
			      ],
			      "chunks": [
			        "chunkB",
			      ],
			      "emitted": true,
			      "info": {
			        "development": false,
			        "hotModuleReplacement": false,
			      },
			      "name": "chunkB.css",
			      "size": 0,
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
			      "name": "entryA.css",
			      "size": 0,
			      "type": "asset",
			    },
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
			      "name": "entryB.css",
			      "size": 0,
			      "type": "asset",
			    },
			  ],
			  "assetsByChunkName": {
			    "chunkB": [
			      "chunkB.css",
			      "chunkB.js",
			    ],
			    "entryA": [
			      "entryA.css",
			      "entryA.js",
			    ],
			    "entryB": [
			      "entryB.css",
			      "entryB.js",
			    ],
			  },
			  "errorsCount": 1,
			}
		`);
		});
	});
});
