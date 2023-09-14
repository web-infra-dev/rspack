import * as util from "util";
import path from "path";
import { rspack, RspackOptions, Stats } from "../src";
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
		expect(stats?.toJson(statsOptions)).toMatchSnapshot();
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
		"PublicPath: auto
		asset main.js 215 bytes {main} [emitted] (name: main)
		Entrypoint main 215 bytes = main.js
		chunk {main} main.js (main) [entry]
		  ./fixtures/a.js [876] {main}
		    entry ./fixtures/a
		./fixtures/a.js [876] {main}
		  entry ./fixtures/a
		rspack compiled successfully (224575f0bba4bc71b95f)"
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
		asset main.js 419 bytes [emitted] (name: main)
		Entrypoint main 419 bytes = main.js
		./fixtures/a.js
		./fixtures/b.js
		./fixtures/c.js
		./fixtures/abc.js

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



		rspack compiled with 1 error (51da9544767033575b9e)"
	`);
	});

	it("should output stats with query", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/abc-query"
		});

		const statsOptions = {
			all: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(stats?.toJson(statsOptions)).toMatchSnapshot();
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
		<t> make hook: X ms
		<t> module add task: X ms
		<t> module process dependencies task: X ms
		<t> module factorize task: X ms
		<t> module build task: X ms
		<t> finish modules: X ms
		<t> optimize dependencies: X ms
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
		"./fixtures/a.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/b.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/c.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)
		./fixtures/abc.js
		  X ms (resolving: X ms, integration: X ms, building: X ms)"
	`);
	});

	it("should have cache hits log when logging verbose and cache is enabled", async () => {
		const compiler = rspack({
			context: __dirname,
			entry: "./fixtures/abc",
			cache: true,
			experiments: {
				incrementalRebuild: false
			}
		});
		await new Promise<void>((resolve, reject) => {
			compiler.build(err => {
				if (err) {
					return reject(err);
				}
				resolve();
			});
		});
		const stats = await new Promise<string>((resolve, reject) => {
			compiler.rebuild(
				new Set([path.join(__dirname, "./fixtures/a")]),
				new Set(),
				err => {
					if (err) {
						return reject(err);
					}
					const stats = new Stats(compiler.compilation).toString({
						all: false,
						logging: "verbose"
					});
					resolve(stats);
				}
			);
		});
		expect(stats).toContain("module build cache: 100.0% (4/4)");
		expect(stats).toContain("module factorize cache: 100.0% (5/5)");
		expect(stats).toContain("module code generation cache: 100.0% (4/4)");
	});

	it("should not have any cache hits log when cache is disabled", async () => {
		const compiler = rspack({
			context: __dirname,
			entry: "./fixtures/abc",
			cache: false,
			experiments: {
				incrementalRebuild: false
			}
		});
		await new Promise<void>((resolve, reject) => {
			compiler.build(err => {
				if (err) {
					return reject(err);
				}
				resolve();
			});
		});
		const stats = await new Promise<string>((resolve, reject) => {
			compiler.rebuild(
				new Set([path.join(__dirname, "./fixtures/a")]),
				new Set(),
				err => {
					if (err) {
						return reject(err);
					}
					const stats = new Stats(compiler.compilation).toString({
						all: false,
						logging: "verbose"
					});
					resolve(stats);
				}
			);
		});
		expect(stats).not.toContain("module build cache");
		expect(stats).not.toContain("module factorize cache");
		expect(stats).not.toContain("module code generation cache");
	});

	it("should have any cache hits log of modules in incremental rebuild mode", async () => {
		const compiler = rspack({
			context: __dirname,
			entry: "./fixtures/abc",
			cache: true,
			experiments: {
				incrementalRebuild: true
			}
		});
		await new Promise<void>((resolve, reject) => {
			compiler.build(err => {
				if (err) {
					return reject(err);
				}
				resolve();
			});
		});
		const stats = await new Promise<string>((resolve, reject) => {
			compiler.rebuild(
				new Set([path.join(__dirname, "./fixtures/a")]),
				new Set(),
				err => {
					if (err) {
						return reject(err);
					}
					const stats = new Stats(compiler.compilation).toString({
						all: false,
						logging: "verbose"
					});
					resolve(stats);
				}
			);
		});
		expect(stats).toContain("module build cache: 100.0% (1/1)");
		expect(stats).toContain("module factorize cache: 100.0% (1/1)");
		expect(stats).toContain("module code generation cache: 100.0% (4/4)");
	});

	it("should have ids when ids is true", async () => {
		const stats = await compile({
			context: __dirname,
			entry: "./fixtures/a"
		});
		const options = {
			all: false,
			assets: true,
			modules: true,
			chunks: true,
			ids: true
		};
		expect(stats?.toJson(options)).toMatchSnapshot();
		expect(stats?.toString(options).replace(/\\/g, "/")).toMatchInlineSnapshot(`
		"asset main.js 215 bytes {main} [emitted] (name: main)
		chunk {main} main.js (main) [entry]
		./fixtures/a.js [876] {main}"
	`);
	});
});
it("should include asset.info.sourceFilename", async () => {
	const stats = await compile({
		context: __dirname,
		entry: "./fixtures/sourceFilename.js",
		module: {
			rules: [
				{
					test: /\.png$/,
					type: "asset/resource"
				}
			]
		}
	});
	expect(stats?.toJson({ assets: true, all: false })).toMatchInlineSnapshot(`
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
				"sourceFilename": "fixtures/empty.png",
			},
			"name": "487c3f5e2b0d6a79324e.png",
			"size": 0,
			"type": "asset",
			},
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
			"size": 751,
			"type": "asset",
			},
		],
		"assetsByChunkName": {
			"main": [
			"487c3f5e2b0d6a79324e.png",
			"main.js",
			],
		},
		}
		`);
});
