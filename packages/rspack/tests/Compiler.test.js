"use strict";

require("./helpers/warmup-webpack");

const path = require("path");
// CHANGE: changed the import path
const { Stats } = require("..");
const { createFsFromVolume, Volume } = require("memfs");
const captureStdio = require("./helpers/captureStdio");
const deprecationTracking = require("./helpers/deprecationTracking");

describe("Compiler", () => {
	jest.setTimeout(20000);
	function compile(entry, options, callback) {
		const noOutputPath = !options.output || !options.output.path;
		const webpack = require("..");
		options = webpack.config.getNormalizedWebpackOptions(options);
		if (!options.mode) options.mode = "production";
		options.entry = entry;
		options.context = path.join(__dirname, "fixtures");
		if (noOutputPath) options.output.path = "/";
		// CHANGE: The pathinfo is currently not supported in rspack
		// options.output.pathinfo = true;
		options.optimization = {
			minimize: false
		};
		const logs = {
			mkdir: [],
			writeFile: []
		};
		const c = webpack(options);
		const files = {};
		c.outputFileSystem = {
			// CHANGE: Added support for the `options` parameter to enable recursive directory creation,
			// accommodating Rspack's requirement that differs from webpack's usage
			mkdir(path, options, callback) {
				let recursive = false;
				if (typeof options === "function") {
					callback = options;
				} else if (options) {
					if (options.recursive !== undefined) recursive = options.recursive;
				}
				logs.mkdir.push(path);
				if (recursive) {
					callback();
				} else {
					const err = new Error();
					err.code = "EEXIST";
					callback(err);
				}
			},
			writeFile(name, content, callback) {
				logs.writeFile.push(name, content);
				files[name] = content.toString("utf-8");
				callback();
			},
			stat(path, callback) {
				callback(new Error("ENOENT"));
			}
		};
		c.hooks.compilation.tap(
			"CompilerTest",
			compilation => (compilation.bail = true)
		);
		c.run((err, stats) => {
			if (err) throw err;
			expect(typeof stats).toBe("object");
			const compilation = stats.compilation;
			stats = stats.toJson({
				modules: true,
				reasons: true
			});
			expect(typeof stats).toBe("object");
			expect(stats).toHaveProperty("errors");
			expect(Array.isArray(stats.errors)).toBe(true);
			if (stats.errors.length > 0) {
				expect(stats.errors[0]).toBeInstanceOf(Error);
				throw stats.errors[0];
			}
			stats.logs = logs;
			c.close(err => {
				if (err) return callback(err);
				callback(stats, files, compilation);
			});
		});
	}

	let compiler;
	afterEach(callback => {
		if (compiler) {
			compiler.close(callback);
			compiler = undefined;
		} else {
			callback();
		}
	});

	it("should bubble up errors when wrapped in a promise and bail is true (empty dependency)", async () => {
		try {
			const createCompiler = options => {
				return new Promise((resolve, reject) => {
					const webpack = require("..");
					const c = webpack(options);
					c.run((err, stats) => {
						if (err) {
							reject(err);
						}
						if (stats !== undefined && "errors" in stats) {
							reject(err);
						} else {
							resolve(c);
						}
					});
					return c;
				});
			};
			compiler = await createCompiler({
				context: path.join(__dirname, "fixtures"),
				mode: "production",
				entry: "./empty-dependency",
				output: {
					filename: "bundle.js"
				},
				bail: true
			});
		} catch (err) {
			expect(err.toString()).toMatchInlineSnapshot(`
			"Error:   × Empty dependency: Expected a non-empty request
			   ╭─[1:1]
			 1 │ module.exports = function b() {
			 2 │     /* eslint-disable node/no-missing-require */ require("");
			   ·                                                  ───────────
			 3 │     return "This is an empty dependency";
			 4 │ };
			   ╰────
			"
		`);
		}
	});

	describe("infrastructure logging", () => {
		let capture;
		beforeEach(() => {
			capture = captureStdio(process.stderr);
		});
		afterEach(() => {
			capture.restore();
		});
		const escapeAnsi = stringRaw =>
			stringRaw
				.replace(/\u001b\[1m\u001b\[([0-9;]*)m/g, "<CLR=$1,BOLD>")
				.replace(/\u001b\[1m/g, "<CLR=BOLD>")
				.replace(/\u001b\[39m\u001b\[22m/g, "</CLR>")
				.replace(/\u001b\[([0-9;]*)m/g, "<CLR=$1>");
		class MyPlugin {
			apply(compiler) {
				const logger = compiler.getInfrastructureLogger("MyPlugin");
				logger.time("Time");
				logger.group("Group");
				logger.error("Error");
				logger.warn("Warning");
				logger.info("Info");
				logger.log("Log");
				logger.debug("Debug");
				logger.groupCollapsed("Collapsed group");
				logger.log("Log inside collapsed group");
				logger.groupEnd();
				logger.groupEnd();
				logger.timeEnd("Time");
			}
		}

		it("should print error with stack information with async callback", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.processAssets.tapPromise(
							"MyPlugin",
							async assets => {
								throw new Error("Failed to handle process assets from JS");
							}
						);
					});
				}
			}
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					filename: "bundle.js"
				},
				plugins: [new MyPlugin()]
			});

			compiler.run((err, stats) => {
				expect(
					err.message.includes("Failed to handle process assets from JS")
				).toBeTruthy();
				done();
			});
		});
		it("should print error with stack information with sync callback", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						throw new Error("Failed to handle process assets from JS");
					});
				}
			}
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					filename: "bundle.js"
				},
				plugins: [new MyPlugin()]
			});

			compiler.run((err, stats) => {
				expect(
					err.message.includes("Failed to handle process assets from JS")
				).toBeTruthy();
				done();
			});
		});
	});
	describe("compilation", () => {
		it("should be called", done => {
			const mockFn = jest.fn();
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						mockFn();
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build(() => {
				compiler.build(() => {
					expect(mockFn).toBeCalledTimes(2);
					done();
				});
			});
		});

		it("should work with `namedChunks`", done => {
			const mockFn = jest.fn();
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.afterCompile.tap("Plugin", compilation => {
						let c = compilation.namedChunks.get("d");
						expect(c.name).toBe("d");
						mockFn();
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: {
					d: "./d"
				},
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build(error => {
				expect(error).toBeFalsy();
				expect(mockFn).toBeCalled();
				done();
			});
		});

		it("should get assets with both `getAssets` and `assets`(getter)", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							let list = compilation.getAssets();
							let map = compilation.assets;

							expect(Object.keys(map)).toHaveLength(list.length);

							list.forEach(a => {
								const b = map[a.name];
								expect(a.source.buffer()).toEqual(b.buffer());
							});
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build((err, stats) => {
				done(err);
			});
		});

		it("should update assets", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							const oldSource = compilation.assets["main.js"];
							expect(oldSource).toBeTruthy();
							expect(oldSource.source().includes("This is d")).toBeTruthy();
							const { RawSource } = require("webpack-sources");
							const updatedSource = new RawSource(
								`module.exports = "This is the updated d"`
							);
							compilation.updateAsset(
								"main.js",
								source => {
									expect(source.buffer()).toEqual(oldSource.buffer());
									return updatedSource;
								},
								_ => _
							);

							const newSource = compilation.assets["main.js"];
							expect(newSource).toBeTruthy();
							expect(newSource.buffer()).toStrictEqual(updatedSource.buffer());
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build((err, stats) => {
				done(err);
			});
		});

		it("should throw if the asset to be updated is not exist", done => {
			const mockFn = jest.fn();

			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							const { RawSource } = require("webpack-sources");
							try {
								compilation.updateAsset(
									"something-else.js",
									new RawSource(`module.exports = "something-else"`),
									{
										minimized: true,
										development: true,
										related: {},
										hotModuleReplacement: false
									}
								);
							} catch (err) {
								mockFn();
								expect(err).toMatchInlineSnapshot(
									`[Error: Called Compilation.updateAsset for not existing filename something-else.js]`
								);
							}
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build((err, stats) => {
				expect(mockFn).toHaveBeenCalled();

				done(err);
			});
		});

		it("should emit assets correctly", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						let assets = compilation.getAssets();
						expect(assets.length).toBe(0);
						const { RawSource } = require("webpack-sources");
						compilation.emitAsset(
							"dd.js",
							new RawSource(`module.exports = "This is dd"`)
						);
						compilation.hooks.processAssets.tap("Plugin", assets => {
							let names = Object.keys(assets);

							expect(names.length).toBe(2); // ["main.js", "dd.js"]
							expect(names.includes("main.js")).toBeTruthy();
							expect(assets["main.js"].source().includes("This is d"));

							expect(names.includes("dd.js")).toBeTruthy();
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()],
				output: {
					path: "/directory"
				}
			});

			const outputFileSystem = createFsFromVolume(new Volume());
			compiler.outputFileSystem = outputFileSystem;

			compiler.build((err, stats) => {
				if (err) {
					return done(err);
				}

				if (
					outputFileSystem.existsSync("/directory/main.js") &&
					outputFileSystem.existsSync("/directory/dd.js")
				) {
					const dd = outputFileSystem.readFileSync("/directory/dd.js", "utf-8");

					if (dd !== `module.exports="This is dd";`) {
						return done(new Error("File content is not correct"));
					}

					return done();
				}

				done(new Error("File not found"));
			});
		});

		it("should have error if the asset to be emitted is exist", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							const { RawSource } = require("webpack-sources");
							compilation.emitAsset(
								"main.js",
								new RawSource(`module.exports = "I'm the right main.js"`)
							);
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build(err => {
				const stats = new Stats(compiler.compilation);
				expect(stats.toJson().errors[0].message).toMatchInlineSnapshot(`
			"  × Conflict: Multiple assets emit different content to the same filename main.js
			"
		`);
				done();
			});
		});

		it("should call optimizeModules hook correctly", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.optimizeModules.tap("MyPlugin", modules => {
							expect(modules.length).toEqual(1);
							expect(modules[0].resource.includes("d.js")).toBeTruthy();
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build(err => {
				done(err);
			});
		});

		it("should call afterOptimizeModules hook correctly", done => {
			class MyPlugin {
				apply(compiler) {
					let a = 1;
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.optimizeModules.tap("MyPlugin", () => {
							a += 1;
						});

						compilation.hooks.afterOptimizeModules.tap("MyPlugin", modules => {
							expect(a).toBeGreaterThan(1);
							expect(modules.length).toEqual(1);
							expect(modules[0].resource.includes("d.js")).toBeTruthy();
						});
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build(err => {
				done(err);
			});
		});

		it("should call getCache function correctly", done => {
			class MyPlugin {
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						let cache = compilation.getCache("MyPlugin");
						expect(cache).not.toBeNull();
					});
				}
			}
			const webpack = require("..");
			const compiler = webpack({
				entry: "./d",
				context: path.join(__dirname, "fixtures"),
				plugins: [new MyPlugin()]
			});

			compiler.build(err => {
				done(err);
			});
		});
	});

	describe("should print error", () => {
		it("splitChunks.minChunks equals 0", done => {
			const webpack = require("..");
			try {
				webpack({
					entry: "./a",
					context: path.join(__dirname, "fixtures"),
					optimization: {
						splitChunks: {
							minChunks: 0
						}
					}
				});
			} catch (err) {
				expect(err.toString()).toContain(
					'Number must be greater than or equal to 1 at "optimization.splitChunks.minChunks"'
				);
				done();
			}

			expect.assertions(1);
		});
	});
});
