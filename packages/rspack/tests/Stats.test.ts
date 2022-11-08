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
		expect(stats.toJson()).toMatchInlineSnapshot(`
		{
		  "assets": [
		    {
		      "chunkNames": [],
		      "chunks": [],
		      "info": {
		        "development": false,
		      },
		      "name": "runtime.js",
		      "size": 14839,
		      "type": "asset",
		    },
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
		      "size": 210,
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
		expect(stats.toString({ colors: false })).toMatchInlineSnapshot(`
		"     Asset       Size  Chunks  Chunk Names
		runtime.js   14.5 KiB          
		   main.js  210 bytes    main  main
		chunk {main} main.js (main) 55 bytes [entry]
		[./fixtures/a.js] 55 bytes {main}"
	`);
	});
});
