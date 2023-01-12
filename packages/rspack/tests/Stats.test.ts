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
		const statsOptions = { all: true };
		expect(typeof stats.hash).toBe("string");
		expect(stats.toJson(statsOptions)).toMatchInlineSnapshot(`
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
		  "chunks": [
		    {
		      "entry": true,
		      "files": [
		        "main.js",
		      ],
		      "id": "main",
		      "initial": true,
		      "names": [
		        "main",
		      ],
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
		  "hash": "9f3a9462bb595239",
		  "modules": [
		    {
		      "chunks": [
		        "main",
		      ],
		      "id": "876",
		      "identifier": "<PROJECT_ROOT>/tests/fixtures/a.js",
		      "issuerPath": [],
		      "moduleType": "js",
		      "name": "./fixtures/a.js",
		      "reasons": [
		        {},
		      ],
		      "size": 55,
		      "type": "module",
		    },
		  ],
		  "warnings": [],
		  "warningsCount": 0,
		}
	`);
		expect(stats.toString(statsOptions)).toMatchInlineSnapshot(`
		"Hash: 9f3a9462bb595239
		  Asset       Size  Chunks             Chunk Names
		main.js  215 bytes    main  [emitted]  main
		Entrypoint main = main.js
		chunk {main} main.js (main) 55 bytes [entry]
		[876] ./fixtures/a.js 55 bytes {main}
		    "
	`);
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

	it("should look not bad for default stats toString", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/abc"
		});
		expect(stats.toString()).toMatchInlineSnapshot(`
		"Hash: 3a2af34d427feeeb
		  Asset       Size  Chunks             Chunk Names
		main.js  419 bytes    main  [emitted]  main
		Entrypoint main = main.js
		[876] ./fixtures/a.js 55 bytes {main}
		[211] ./fixtures/b.js 94 bytes {main}
		[537] ./fixtures/c.js 72 bytes {main}
		[222] ./fixtures/abc.js 83 bytes {main}

		error[javascript]: JavaScript parsing error
		  ┌─ tests/fixtures/b.js:6:1
		  │
		6 │ return;
		  │ ^^^^^^^ Return statement is not allowed here

		"
	`);
	});
});
