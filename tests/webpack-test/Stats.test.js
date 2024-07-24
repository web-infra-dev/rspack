"use strict";

require("./helpers/warmup-webpack");

const { createFsFromVolume, Volume } = require("memfs");
const { normalizeFilteredTestName, FilteredStatus } = require("./lib/util/filterUtil");

const compile = options => {
	return new Promise((resolve, reject) => {
		const webpack = require("@rspack/core");
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
	it("should print env string in stats", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/a"
		});
		expect(
			stats.toString({
				all: false,
				env: true,
				_env: "production"
			})
		).toBe('Environment (--env): "production"');
		expect(
			stats.toString({
				all: false,
				env: true,
				_env: {
					prod: ["foo", "bar"],
					baz: true
				}
			})
		).toBe(
			"Environment (--env): {\n" +
			'  "prod": [\n' +
			'    "foo",\n' +
			'    "bar"\n' +
			"  ],\n" +
			'  "baz": true\n' +
			"}"
		);
	});
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
		// CHANGE: skipped as rspack generates additional chunk
		it.skip(normalizeFilteredTestName(FilteredStatus.TODO, "should be empty when there is no additional chunks"), async () => {
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
		it.skip(normalizeFilteredTestName(FilteredStatus.TODO, "should contain additional chunks"), async () => {
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
			          "name": "entryB.js",
			          "size": 4399,
			        },
			      ],
			      "assetsSize": 4399,
			      "chunks": [
			        "entryB",
			      ],
			      "name": "entryB",
			    },
			  },
			}
		`);
		});
		it.skip(normalizeFilteredTestName(FilteredStatus.TODO, "should contain assets"), async () => {
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
			      "size": 4399,
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
			      "size": 215,
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
			  ],
			  "assetsByChunkName": {
			    "chunkB": [
			      "chunkB.js",
			    ],
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
