import * as util from "util";
import { rspack, RspackOptions } from "../src";
import serializer from "jest-serializer-path";

expect.addSnapshotSerializer(serializer);

const compile = async (options: RspackOptions) => {
	return util.promisify(rspack)(options);
};

describe("Stats", () => {
	it("should have stats", async () => {
		const stats = await compile({
			context: __dirname,
			entry: {
				main: "./fixtures/a"
			}
		});
		const statsOptions = { all: true, timings: false, builtAt: false };
		expect(typeof stats?.hash).toBe("string");
		expect(stats?.toJson(statsOptions)).toMatchInlineSnapshot(`
		{
		  "assets": [
		    {
		      "chunkNames": [
		        "main",
		      ],
		      "chunks": [
		        "main",
		      ],
		      "emitted": true,
		      "info": {
		        "development": false,
		        "hotModuleReplacement": false,
		      },
		      "name": "main.js",
		      "size": 215,
		      "type": "asset",
		    },
		  ],
		  "assetsByChunkName": {
		    "main": [
		      "main.js",
		    ],
		  },
		  "chunks": [
		    {
		      "children": [],
		      "entry": true,
		      "files": [
		        "main.js",
		      ],
		      "id": "main",
		      "initial": true,
		      "modules": [
		        {
		          "chunks": [
		            "main",
		          ],
		          "id": "658",
		          "identifier": "javascript/dynamic|<PROJECT_ROOT>/tests/fixtures/a.js",
		          "issuerPath": [],
		          "moduleType": "javascript/dynamic",
		          "name": "./fixtures/a.js",
		          "size": 55,
		          "type": "module",
		        },
		      ],
		      "names": [
		        "main",
		      ],
		      "parents": [],
		      "siblings": [],
		      "size": 55,
		      "type": "chunk",
		    },
		  ],
		  "entrypoints": {
		    "main": {
		      "assets": [
		        {
		          "name": "main.js",
		          "size": 215,
		        },
		      ],
		      "assetsSize": 215,
		      "chunks": [
		        "main",
		      ],
		      "name": "main",
		    },
		  },
		  "errors": [],
		  "errorsCount": 0,
		  "hash": "ff293361e645d785",
		  "modules": [
		    {
		      "chunks": [
		        "main",
		      ],
		      "id": "658",
		      "identifier": "javascript/dynamic|<PROJECT_ROOT>/tests/fixtures/a.js",
		      "issuerPath": [],
		      "moduleType": "javascript/dynamic",
		      "name": "./fixtures/a.js",
		      "size": 55,
		      "type": "module",
		    },
		  ],
		  "namedChunkGroups": {
		    "main": {
		      "assets": [
		        {
		          "name": "main.js",
		          "size": 215,
		        },
		      ],
		      "assetsSize": 215,
		      "chunks": [
		        "main",
		      ],
		      "name": "main",
		    },
		  },
		  "outputPath": "<PROJECT_ROOT>/dist",
		  "publicPath": "auto",
		  "warnings": [],
		  "warningsCount": 0,
		}
	`);
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
		"Hash: ff293361e645d785
		PublicPath: auto
		  Asset       Size  Chunks             Chunk Names
		main.js  215 bytes    main  [emitted]  main
		Entrypoint main = main.js
		chunk {main} main.js (main) 55 bytes [entry]
		 [658] ./fixtures/a.js 55 bytes {main}
		[658] ./fixtures/a.js 55 bytes {main}"
	`);
	});

	it("should omit all properties with all false", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/a"
		});
		expect(
			stats?.toJson({
				all: false
			})
		).toEqual({});
	});

	it("should look not bad for default stats toString", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/abc"
		});
		expect(stats?.toString({ timings: false })).toMatchInlineSnapshot(`
		"Hash: 2168fece27972fed
		PublicPath: auto
		  Asset       Size  Chunks             Chunk Names
		main.js  419 bytes    main  [emitted]  main
		Entrypoint main = main.js
		[658] ./fixtures/a.js 55 bytes {main}
		[385] ./fixtures/b.js 94 bytes {main}
		[586] ./fixtures/c.js 72 bytes {main}
		[939] ./fixtures/abc.js 83 bytes {main}

		error[javascript]: JavaScript parsing error
		  ┌─ tests/fixtures/b.js:6:1
		  │
		6 │ return;
		  │ ^^^^^^^ Return statement is not allowed here

		"
	`);
	});

	it("should have reasons", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/main5.js",
			stats: { reasons: true }
		});
		expect(stats?.toJson({}, true).modules).toEqual([
			{
				chunks: ["main"],
				id: "658",
				identifier:
					"javascript/dynamic|/Users/faga/f/rspack/packages/rspack/tests/fixtures/a.js",
				issuer:
					"javascript/dynamic|/Users/faga/f/rspack/packages/rspack/tests/fixtures/main5.js",
				issuerId: "597",
				issuerName: "./fixtures/main5.js",
				issuerPath: [
					{
						id: "597",
						identifier:
							"javascript/dynamic|/Users/faga/f/rspack/packages/rspack/tests/fixtures/main5.js",
						name: "./fixtures/main5.js"
					}
				],
				moduleType: "javascript/dynamic",
				name: "./fixtures/a.js",
				reasons: [
					{
						moduleId: "597",
						moduleIdentifier:
							"javascript/dynamic|/Users/faga/f/rspack/packages/rspack/tests/fixtures/main5.js",
						moduleName: "./fixtures/main5.js",
						type: "cjs require",
						userRequest: "./a"
					}
				],
				size: 55,
				type: "module"
			},
			{
				chunks: ["main"],
				id: "597",
				identifier:
					"javascript/dynamic|/Users/faga/f/rspack/packages/rspack/tests/fixtures/main5.js",
				issuerPath: [],
				moduleType: "javascript/dynamic",
				name: "./fixtures/main5.js",
				reasons: [{ type: "entry", userRequest: "./fixtures/main5.js" }],
				size: 44,
				type: "module"
			}
		]);
	});
});
