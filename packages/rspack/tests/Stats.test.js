"use strict";

require("./helpers/warmup-webpack");

const { createFsFromVolume, Volume } = require("memfs");
const path = require("path");
const { Stats } = require("../dist");
const serializer = require("jest-serializer-path");

expect.addSnapshotSerializer(serializer);

const compile = options => {
	return new Promise((resolve, reject) => {
		const webpack = require("..");
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
asset main.js 211 bytes {909} [emitted] (name: main)
Entrypoint main 211 bytes = main.js
chunk {909} main.js (main) [entry]
  ./fixtures/a.js [585] {909}
    [no exports]
    [used exports unknown]
    entry ./fixtures/a
./fixtures/a.js [585] {909}
  [no exports]
  [used exports unknown]
  entry ./fixtures/a
  
Rspack compiled successfully (57e46af248a1c1fe076f)"
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
		expect(stats?.toString({ timings: false, version: false }))
			.toMatchInlineSnapshot(`
"PublicPath: auto
asset main.js 738 bytes [emitted] (name: main)
Entrypoint main 738 bytes = main.js
./fixtures/a.js
./fixtures/b.js
./fixtures/c.js
./fixtures/abc.js

ERROR in ./fixtures/b.js
  × Module parse failed:
  ╰─▶   × JavaScript parsing error: Return statement is not allowed here
         ╭─[4:1]
       4 │
       5 │ // Test CJS top-level return
       6 │ return;
         · ───────
         ╰────
      
  help: 
        You may need an appropriate loader to handle this file type.

Rspack compiled with 1 error (8137ab425c2721784808)"
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
				.replace(/\d+ ms/g, "X ms")
		).toMatchInlineSnapshot(`
		"LOG from rspack.Compilation
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
		<t> optimize code generation: X ms
		<t> code generation: X ms
		<t> runtime requirements.modules: X ms
		<t> runtime requirements.chunks: X ms
		<t> runtime requirements.entries: X ms
		<t> runtime requirements: X ms
		<t> hashing: hash chunks: X ms
		<t> hashing: hash runtime chunks: X ms
		<t> hashing: process full hash chunks: X ms
		<t> hashing: X ms
		<t> create module assets: X ms
		<t> create chunk assets: X ms
		<t> process assets: X ms
		<t> after process assets: X ms

		LOG from rspack.Compiler
		<t> make hook: X ms
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

		LOG from rspack.WarnCaseSensitiveModulesPlugin
		<t> check case sensitive modules: X ms

		LOG from rspack.buildChunkGraph
		<t> prepare entrypoints: X ms
		<t> process queue: X ms
		<t> extend chunkGroup runtime: X ms
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
			stats?.toString({ all: false, modules: true }).replace(/\d+ ms/g, "X ms")
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

	it("should not have any cache hits log when cache is disabled", async () => {
		const compiler = require("../dist")({
			context: __dirname,
			entry: "./fixtures/abc",
			cache: false
		});
		await new Promise((resolve, reject) => {
			compiler.build(err => {
				if (err) {
					return reject(err);
				}
				resolve();
			});
		});
		const stats = await new Promise((resolve, reject) => {
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
		const compiler = require("../dist")({
			context: __dirname,
			entry: "./fixtures/abc",
			cache: true
		});
		await new Promise((resolve, reject) => {
			compiler.build(err => {
				if (err) {
					return reject(err);
				}
				resolve();
			});
		});
		const stats = await new Promise((resolve, reject) => {
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
		expect(stats?.toString(options)).toMatchInlineSnapshot(`
		"asset main.js 211 bytes {909} [emitted] (name: main)
		chunk {909} main.js (main) [entry]
		./fixtures/a.js [585] {909}"
	`);
	});

	it("should have null as placeholders in stats before chunkIds", async () => {
		let stats;

		class TestPlugin {
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("custom", compilation => {
					compilation.hooks.optimizeModules.tap("test plugin", () => {
						stats = compiler.compilation.getStats().toJson({});
					});
				});
			}
		}
		await compile({
			context: __dirname,
			entry: "./fixtures/a",
			plugins: [new TestPlugin()]
		});

		expect(stats.entrypoints).toMatchInlineSnapshot(`
		{
		  "main": {
		    "assets": [],
		    "assetsSize": 0,
		    "chunks": [
		      null,
		    ],
		    "name": "main",
		  },
		}
	`);
	});

	it("should have children when using childCompiler", async () => {
		let statsJson;

		class TestPlugin {
			apply(compiler) {
				compiler.hooks.thisCompilation.tap(TestPlugin.name, compilation => {
					compilation.hooks.processAssets.tapAsync(
						TestPlugin.name,
						async (assets, callback) => {
							const child = compiler.createChildCompiler(
								compilation,
								"TestChild",
								1,
								compilation.outputOptions,
								[
									new compiler.webpack.EntryPlugin(
										compiler.context,
										"./fixtures/abc",
										{ name: "TestChild" }
									)
								]
							);
							child.runAsChild(err => callback(err));
						}
					);
				});
				compiler.hooks.done.tap("test plugin", stats => {
					statsJson = stats.toJson({
						all: false,
						children: true,
						assets: true
					});
				});
			}
		}
		await compile({
			context: __dirname,
			entry: "./fixtures/a",
			plugins: [new TestPlugin()]
		});

		expect(statsJson).toMatchInlineSnapshot(`
		{
		  "assets": [
		    {
		      "chunkNames": [],
		      "emitted": true,
		      "info": {
		        "development": false,
		        "hotModuleReplacement": false,
		      },
		      "name": "TestChild.js",
		      "size": 738,
		      "type": "asset",
		    },
		    {
		      "chunkNames": [
		        "main",
		      ],
		      "emitted": true,
		      "info": {
		        "development": false,
		        "hotModuleReplacement": false,
		      },
		      "name": "main.js",
		      "size": 211,
		      "type": "asset",
		    },
		  ],
		  "assetsByChunkName": {
		    "main": [
		      "main.js",
		    ],
		  },
		  "children": [
		    {
		      "assets": [
		        {
		          "chunkNames": [
		            "TestChild",
		          ],
		          "emitted": true,
		          "info": {
		            "development": false,
		            "hotModuleReplacement": false,
		          },
		          "name": "TestChild.js",
		          "size": 738,
		          "type": "asset",
		        },
		      ],
		      "assetsByChunkName": {
		        "TestChild": [
		          "TestChild.js",
		        ],
		      },
		      "children": [],
		      "name": "TestChild",
		    },
		  ],
		}
	`);
	});

	it("should have usedExports and providedExports stats", async () => {
		const stats = await compile({
			context: __dirname,
			entry: {
				main: "./fixtures/esm/abc"
			},
			optimization: {
				usedExports: true,
				providedExports: true
			},
			experiments: {
				rspackFuture: {
					newTreeshaking: true
				}
			}
		});
		const statsOptions = {
			usedExports: true,
			providedExports: true,
			timings: false,
			builtAt: false,
			version: false
		};
		expect(typeof stats?.hash).toBe("string");
		expect(stats?.toJson(statsOptions)).toMatchSnapshot();
		expect(stats?.toString(statsOptions)).toMatchInlineSnapshot(`
		"PublicPath: auto
		asset main.js 784 bytes [emitted] (name: main)
		Entrypoint main 784 bytes = main.js
		runtime modules 3 modules
		./fixtures/esm/a.js
		  [exports: a, default]
		  [only some exports used: a]
		./fixtures/esm/b.js
		  [exports: b, default]
		  [only some exports used: default]
		./fixtures/esm/c.js
		  [exports: c, default]
		./fixtures/esm/abc.js
		  [no exports]
		  [no exports used]
		Rspack compiled successfully (90855ad020cd8866adbb)"
	`);
	});
});
