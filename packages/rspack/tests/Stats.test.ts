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
		      "chunks": [],
		      "name": "runtime.js",
		      "size": 14874,
		      "type": "asset",
		    },
		    {
		      "chunks": [
		        "main",
		      ],
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
		        "runtime.js",
		      ],
		      "id": "main",
		      "initial": true,
		      "names": [
		        "main",
		      ],
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
		}
	`);
	});
});
