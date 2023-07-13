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
		const statsOptions = {
			all: true,
			timings: false,
			builtAt: false,
			version: false
		};
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
		          "assets": [],
		          "chunks": [
		            "main",
		          ],
		          "id": "876",
		          "identifier": "<PROJECT_ROOT>/tests/fixtures/a.js",
		          "issuerPath": [],
		          "moduleType": "javascript/auto",
		          "name": "./fixtures/a.js",
		          "reasons": [
		            {
		              "type": "entry",
		              "userRequest": "./fixtures/a",
		            },
		          ],
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
		  "hash": "70329d122baabea3ede8",
		  "modules": [
		    {
		      "assets": [],
		      "chunks": [
		        "main",
		      ],
		      "id": "876",
		      "identifier": "<PROJECT_ROOT>/tests/fixtures/a.js",
		      "issuerPath": [],
		      "moduleType": "javascript/auto",
		      "name": "./fixtures/a.js",
		      "reasons": [
		        {
		          "type": "entry",
		          "userRequest": "./fixtures/a",
		        },
		      ],
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
		"PublicPath: auto
		asset main.js 215 bytes {main} [emitted] (name: main)
		Entrypoint main 215 bytes = main.js
		chunk {main} main.js (main) [entry]
		  ./fixtures/a.js [876] {main}
		    entry ./fixtures/a
		./fixtures/a.js [876] {main}
		  entry ./fixtures/a
		rspack compiled successfully (70329d122baabea3ede8)"
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
		expect(
			stats?.toString({ timings: false, version: false }).replace(/\\/g, "/")
		).toMatchInlineSnapshot(`
		"PublicPath: auto
		asset main.js 419 bytes {main} [emitted] (name: main)
		Entrypoint main 419 bytes = main.js
		./fixtures/a.js [876] {main}
		./fixtures/b.js [211] {main}
		./fixtures/c.js [537] {main}
		./fixtures/abc.js [222] {main}

		error[javascript]: JavaScript parsing error
		  ┌─ tests/fixtures/b.js:6:1
		  │
		6 │ return;
		  │ ^^^^^^^ Return statement is not allowed here



		rspack compiled with 1 error (0910afa30d31b543435c)"
	`);
	});
});
