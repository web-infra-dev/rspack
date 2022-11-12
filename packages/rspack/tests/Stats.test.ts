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
		      "info": {
		        "development": false,
		      },
		      "name": "main.js",
		      "size": 14944,
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
		          "size": 14944,
		        },
		      ],
		      "assetsSize": 14944,
		      "chunks": [
		        "main",
		      ],
		      "name": "main",
		    },
		  },
		  "errors": [],
		  "errorsCount": 0,
		  "modules": [
		    {
		      "chunks": [
		        "main",
		      ],
		      "id": "./fixtures/a.js",
		      "identifier": "<PROJECT_ROOT>/tests/fixtures/a.js",
		      "moduleType": "js",
		      "name": "./fixtures/a.js",
		      "size": 55,
		      "type": "module",
		    },
		  ],
		  "warnings": [],
		  "warningsCount": 0,
		}
	`);
		expect(stats.toString(statsOptions)).toMatchInlineSnapshot(`
		"  Asset      Size  Chunks  Chunk Names
		main.js  14.6 KiB    main  main
		Entrypoint main = main.js
		chunk {main} main.js (main) 55 bytes [entry]
		[./fixtures/a.js] 55 bytes {main}"
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
});
