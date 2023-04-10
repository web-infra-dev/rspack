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
		          "id": "777",
		          "identifier": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/a.js",
		          "issuerPath": [],
		          "moduleType": "javascript/auto",
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
		  "hash": "a8535b55b7de03c8",
		  "modules": [
		    {
		      "chunks": [
		        "main",
		      ],
		      "id": "777",
		      "identifier": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/a.js",
		      "issuerPath": [],
		      "moduleType": "javascript/auto",
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
		"Hash: a8535b55b7de03c8
		PublicPath: auto
		  Asset       Size  Chunks             Chunk Names
		main.js  215 bytes    main  [emitted]  main
		Entrypoint main = main.js
		chunk {main} main.js (main) 55 bytes [entry]
		 [777] ./fixtures/a.js 55 bytes {main}
		[777] ./fixtures/a.js 55 bytes {main}"
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
		"Hash: ef19f9b69eb0c2d0
		PublicPath: auto
		  Asset       Size  Chunks             Chunk Names
		main.js  419 bytes    main  [emitted]  main
		Entrypoint main = main.js
		[777] ./fixtures/a.js 55 bytes {main}
		[510] ./fixtures/b.js 94 bytes {main}
		[906] ./fixtures/c.js 72 bytes {main}
		[492] ./fixtures/abc.js 83 bytes {main}

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
		expect(stats?.toJson({ all: false, modules: true }).modules)
			.toMatchInlineSnapshot(`
[
  {
    "chunks": [
      "main",
    ],
    "id": "777",
    "identifier": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/a.js",
    "issuer": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/main5.js",
    "issuerId": "371",
    "issuerName": "./fixtures/main5.js",
    "issuerPath": [
      {
        "id": "371",
        "identifier": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/main5.js",
        "name": "./fixtures/main5.js",
      },
    ],
    "moduleType": "javascript/auto",
    "name": "./fixtures/a.js",
    "reasons": [
      {
        "moduleId": "371",
        "moduleIdentifier": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/main5.js",
        "moduleName": "./fixtures/main5.js",
        "type": "cjs require",
        "userRequest": "./a",
      },
    ],
    "size": 55,
    "type": "module",
  },
  {
    "chunks": [
      "main",
    ],
    "id": "371",
    "identifier": "javascript/auto|<PROJECT_ROOT>/tests/fixtures/main5.js",
    "issuerPath": [],
    "moduleType": "javascript/auto",
    "name": "./fixtures/main5.js",
    "reasons": [
      {
        "type": "entry",
        "userRequest": "./fixtures/main5.js",
      },
    ],
    "size": 44,
    "type": "module",
  },
]
`);
	});
});
