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
		      "auxiliaryFiles": [],
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
		          "source": "module.exports = function a() {
			return "This is a";
		};",
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
		  "filteredModules": undefined,
		  "hash": "b32fac08a5e8721cacff",
		  "logging": {},
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
		      "source": "module.exports = function a() {
			return "This is a";
		};",
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
		rspack compiled successfully (b32fac08a5e8721cacff)"
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
		2 │     return "This is b";
		3 │ };
		4 │ 
		5 │ // Test CJS top-level return
		6 │ return;
		  │ ^^^^^^^ Return statement is not allowed here
		7 │ 



		rspack compiled with 1 error (27ec1c09308b67dcfd6f)"
	`);
	});

	it("should output the specified number of modules when set stats.modulesSpace", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/abc"
		});

		expect(
			stats?.toJson({
				all: true,
				timings: false,
				builtAt: false,
				version: false
			}).modules?.length
		).toBe(4);

		expect(
			stats?.toJson({
				all: true,
				timings: false,
				builtAt: false,
				version: false,
				modulesSpace: 3
			}).modules?.length
			// 2 = 3 - 1 = max - filteredChildrenLineReserved
		).toBe(2);
	});

	it("should have time log when logging verbose", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/abc"
		});
		expect(
			stats
				?.toString({ all: false, logging: "verbose" })
				.replace(/\\/g, "/")
				.replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		"LOG from rspack.Compilation
		<t> finish modules: X ms
		<t> optimize dependencies: X ms
		<t> create chunks: X ms
		<t> optimize: X ms
		<t> module ids: X ms
		<t> chunk ids: X ms
		<t> code generation: X ms
		<t> runtime requirements.modules: X ms
		<t> runtime requirements.chunks: X ms
		<t> runtime requirements.entries: X ms
		<t> runtime requirements: X ms
		<t> hashing: hash chunks: X ms
		<t> hashing: hash runtime chunks: X ms
		<t> hashing: process full hash chunks: X ms
		<t> hashing: X ms
		<t> create chunk assets: X ms
		<t> process assets: X ms

		LOG from rspack.Compiler
		<t> make hook: X ms
		<t> module add task: X ms
		<t> module process dependencies task: X ms
		<t> module factorize task: X ms
		<t> module build task: X ms
		<t> make: X ms
		<t> finish make hook: X ms
		<t> finish compilation: X ms
		<t> seal compilation: X ms
		<t> afterCompile hook: X ms
		<t> emitAssets: X ms
		<t> done hook: X ms

		LOG from rspack.EnsureChunkConditionsPlugin
		<t> ensure chunk conditions: X ms

		LOG from rspack.RealContentHashPlugin
		<t> hash to asset names: X ms

		LOG from rspack.RemoveEmptyChunksPlugin
		<t> remove empty chunks: X ms

		LOG from rspack.SplitChunksPlugin
		<t> prepare module group map: X ms
		<t> ensure min size fit: X ms
		<t> process module group map: X ms
		<t> ensure max size fit: X ms

		LOG from rspack.buildChunkGraph
		<t> prepare entrypoints: X ms
		<t> process queue: X ms
		<t> extend chunkGroup runtime: X ms
		<t> remove parent modules: X ms
		"
	`);
	});

	it("should have module profile when profile is true", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/abc",
			profile: true
		});
		expect(
			stats
				?.toString({ all: false, modules: true })
				.replace(/\\/g, "/")
				.replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		"./fixtures/a.js [876] {main}
		  [222] ->
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/b.js [211] {main}
		  [222] ->
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/c.js [537] {main}
		  [222] ->
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/abc.js [222] {main}
		  X ms (resolving: X ms, integration: X ms, building: X ms)"
	`);
	});
	it("should have module runtime modules when runtimeModules is true", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/chunk-b"
		});
		expect(stats?.toJson({ all: false, modules: true, runtimeModules: true }))
			.toMatchInlineSnapshot(`
{
  "filteredModules": undefined,
  "modules": [
    {
      "chunks": [
        "chunkB",
      ],
      "id": "211",
      "identifier": "<PROJECT_ROOT>/tests/fixtures/b.js",
      "issuer": "<PROJECT_ROOT>/tests/fixtures/chunk-b.js",
      "issuerId": "346",
      "issuerName": "./fixtures/chunk-b.js",
      "issuerPath": [
        {
          "id": "346",
          "identifier": "<PROJECT_ROOT>/tests/fixtures/chunk-b.js",
          "name": "./fixtures/chunk-b.js",
        },
      ],
      "moduleType": "javascript/auto",
      "name": "./fixtures/b.js",
      "size": 94,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "346",
      "identifier": "<PROJECT_ROOT>/tests/fixtures/chunk-b.js",
      "issuerPath": [],
      "moduleType": "javascript/auto",
      "name": "./fixtures/chunk-b.js",
      "size": 85,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/css_loading",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/css_loading",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/load_script",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/load_script",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/public_path",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/public_path",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/ensure_chunk",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/ensure_chunk",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/has_own_property",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/has_own_property",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/jsonp_chunk_loading",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/jsonp_chunk_loading",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/make_namespace_object",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/make_namespace_object",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/load_chunk_with_module",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/load_chunk_with_module",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/define_property_getters",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/define_property_getters",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/create_fake_namespace_object",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/create_fake_namespace_object",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/get_chunk_filename/__webpack_require__.k",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/get_chunk_filename/__webpack_require__.k",
      "size": 160,
      "type": "module",
    },
    {
      "chunks": [
        "main",
      ],
      "id": "",
      "identifier": "webpack/runtime/get_chunk_filename/__webpack_require__.u",
      "issuerPath": [],
      "moduleType": "runtime",
      "name": "webpack/runtime/get_chunk_filename/__webpack_require__.u",
      "size": 160,
      "type": "module",
    },
  ],
}
`);
	});
});
