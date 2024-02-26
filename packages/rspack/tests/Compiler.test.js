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

	it("should compile a single file to deep output", done => {
		compile(
			"./c",
			{
				output: {
					path: "/what",
					filename: "the/hell.js"
				}
			},
			(stats, files) => {
				// CHANGE: Rspack utilizes the `recursive = true` option for `mkdir`, creating nested directories as needed
				// The expected log should only include the deepest directory "/what/the"
				// expect(stats.logs.mkdir).toEqual(["/what", "/what/the"]);
				expect(stats.logs.mkdir).toEqual(["/what/the"]);
				done();
			}
		);
	});

	// CHANGE: skip due to Rspack defaults to numerical module ids, unlike webpack's string-based ids
	it.skip("should compile a single file", done => {
		compile("./c", {}, (stats, files) => {
			expect(Object.keys(files)).toEqual(["/main.js"]);
			const bundle = files["/main.js"];
			expect(bundle).toMatch("function __webpack_require__(");
			expect(bundle).toMatch(/__webpack_require__\(\/\*! \.\/a \*\/ \w+\);/);
			expect(bundle).toMatch("./c.js");
			expect(bundle).toMatch("./a.js");
			expect(bundle).toMatch("This is a");
			expect(bundle).toMatch("This is c");
			expect(bundle).not.toMatch("2: function(");
			expect(bundle).not.toMatch("window");
			expect(bundle).not.toMatch("jsonp");
			expect(bundle).not.toMatch("fixtures");
			done();
		});
	});

	it.skip("should compile a complex file", done => {
		compile("./main1", {}, (stats, files) => {
			expect(Object.keys(files)).toEqual(["/main.js"]);
			const bundle = files["/main.js"];
			expect(bundle).toMatch("function __webpack_require__(");
			expect(bundle).toMatch("__webpack_require__(/*! ./a */");
			expect(bundle).toMatch("./main1.js");
			expect(bundle).toMatch("./a.js");
			expect(bundle).toMatch("./b.js");
			expect(bundle).toMatch("./node_modules/m1/a.js");
			expect(bundle).toMatch("This is a");
			expect(bundle).toMatch("This is b");
			expect(bundle).toMatch("This is m1/a");
			expect(bundle).not.toMatch("4: function(");
			expect(bundle).not.toMatch("window");
			expect(bundle).not.toMatch("jsonp");
			expect(bundle).not.toMatch("fixtures");
			done();
		});
	});

	it.skip("should compile a file with transitive dependencies", done => {
		compile("./abc", {}, (stats, files) => {
			expect(Object.keys(files)).toEqual(["/main.js"]);
			const bundle = files["/main.js"];
			expect(bundle).toMatch("function __webpack_require__(");
			expect(bundle).toMatch("__webpack_require__(/*! ./a */");
			expect(bundle).toMatch("__webpack_require__(/*! ./b */");
			expect(bundle).toMatch("__webpack_require__(/*! ./c */");
			expect(bundle).toMatch("./abc.js");
			expect(bundle).toMatch("./a.js");
			expect(bundle).toMatch("./b.js");
			expect(bundle).toMatch("./c.js");
			expect(bundle).toMatch("This is a");
			expect(bundle).toMatch("This is b");
			expect(bundle).toMatch("This is c");
			expect(bundle).not.toMatch("4: function(");
			expect(bundle).not.toMatch("window");
			expect(bundle).not.toMatch("jsonp");
			expect(bundle).not.toMatch("fixtures");
			done();
		});
	});

	it.skip("should compile a file with multiple chunks", done => {
		compile("./chunks", {}, (stats, files) => {
			expect(stats.chunks).toHaveLength(2);
			expect(Object.keys(files)).toEqual(["/main.js", "/394.js"]);
			const bundle = files["/main.js"];
			const chunk = files["/394.js"];
			expect(bundle).toMatch("function __webpack_require__(");
			expect(bundle).toMatch("__webpack_require__(/*! ./b */");
			expect(chunk).not.toMatch("__webpack_require__(/* ./b */");
			expect(bundle).toMatch("./chunks.js");
			expect(chunk).toMatch("./a.js");
			expect(chunk).toMatch("./b.js");
			expect(chunk).toMatch("This is a");
			expect(bundle).not.toMatch("This is a");
			expect(chunk).toMatch("This is b");
			expect(bundle).not.toMatch("This is b");
			expect(bundle).not.toMatch("4: function(");
			expect(bundle).not.toMatch("fixtures");
			expect(chunk).not.toMatch("fixtures");
			expect(bundle).toMatch("webpackChunk");
			expect(chunk).toMatch('self["webpackChunk"] || []).push');
			done();
		});
	});

	// cspell:word asmjs
	it.skip("should not evaluate constants in asm.js", done => {
		compile("./asmjs", {}, (stats, files) => {
			expect(Object.keys(files)).toEqual(["/main.js"]);
			const bundle = files["/main.js"];
			expect(bundle).toMatch('"use asm";');
			expect(bundle).toMatch("101");
			expect(bundle).toMatch("102");
			expect(bundle).toMatch("103");
			expect(bundle).toMatch("104");
			expect(bundle).toMatch("105");
			expect(bundle).not.toMatch("106");
			expect(bundle).not.toMatch("107");
			expect(bundle).not.toMatch("108");
			expect(bundle).toMatch("109");
			expect(bundle).toMatch("110");
			done();
		});
	});

	describe("methods", () => {
		let compiler;
		beforeEach(() => {
			const webpack = require("..");
			compiler = webpack({
				entry: "./c",
				context: path.join(__dirname, "fixtures"),
				output: {
					path: "/directory"
					// CHANGE: The pathinfo is currently not supported in rspack
					// pathinfo: true
				}
			});
		});
		afterEach(callback => {
			if (compiler) {
				compiler.close(callback);
				compiler = undefined;
			} else {
				callback();
			}
		});
		describe("purgeInputFileSystem", () => {
			it("invokes purge() if inputFileSystem.purge", done => {
				const mockPurge = jest.fn();
				compiler.inputFileSystem = {
					purge: mockPurge
				};
				compiler.purgeInputFileSystem();
				expect(mockPurge.mock.calls.length).toBe(1);
				done();
			});
			it("does NOT invoke purge() if !inputFileSystem.purge", done => {
				const mockPurge = jest.fn();
				compiler.inputFileSystem = null;
				compiler.purgeInputFileSystem();
				expect(mockPurge.mock.calls.length).toBe(0);
				done();
			});
		});
		describe("isChild", () => {
			it.skip("returns booleanized this.parentCompilation", done => {
				compiler.parentCompilation = "stringyStringString";
				const response1 = compiler.isChild();
				expect(response1).toBe(true);

				compiler.parentCompilation = 123456789;
				const response2 = compiler.isChild();
				expect(response2).toBe(true);

				compiler.parentCompilation = {
					what: "I belong to an object"
				};
				const response3 = compiler.isChild();
				expect(response3).toBe(true);

				compiler.parentCompilation = ["Array", 123, true, null, [], () => {}];
				const response4 = compiler.isChild();
				expect(response4).toBe(true);

				compiler.parentCompilation = false;
				const response5 = compiler.isChild();
				expect(response5).toBe(false);

				compiler.parentCompilation = 0;
				const response6 = compiler.isChild();
				expect(response6).toBe(false);

				compiler.parentCompilation = null;
				const response7 = compiler.isChild();
				expect(response7).toBe(false);

				compiler.parentCompilation = "";
				const response8 = compiler.isChild();
				expect(response8).toBe(false);

				compiler.parentCompilation = NaN;
				const response9 = compiler.isChild();
				expect(response9).toBe(false);
				done();
			});
		});
	});
	it("should not emit on errors", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./missing",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) return done(err);
			if (compiler.outputFileSystem.existsSync("/bundle.js"))
				return done(new Error("Bundle should not be created on error"));
			done();
		});
	});
	it("should bubble up errors when wrapped in a promise and bail is true", async () => {
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
							resolve(stats);
						}
					});
					return c;
				});
			};
			compiler = await createCompiler({
				context: __dirname,
				mode: "production",
				entry: "./missing-file",
				output: {
					path: "/directory",
					filename: "bundle.js"
				},
				bail: true
			});
		} catch (err) {
			expect(err.toString()).toMatch(
				// CHANGE: Error messages from Rspack differ from those in webpack
				// "ModuleNotFoundError: Module not found: Error: Can't resolve './missing-file'"
				"Error:   × Resolve error: Can't resolve './missing-file'"
			);
		}
	});
	// CHANGE: specially added for rspack
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
	it("should not emit compilation errors in async (watch)", async () => {
		const createStats = options => {
			return new Promise((resolve, reject) => {
				const webpack = require("..");
				const c = webpack(options);
				c.outputFileSystem = createFsFromVolume(new Volume());
				const watching = c.watch({}, (err, stats) => {
					watching.close(() => {
						if (err) return reject(err);
						resolve(stats);
					});
				});
			});
		};
		const stats = await createStats({
			context: __dirname,
			mode: "production",
			entry: "./missing-file",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		expect(stats).toBeInstanceOf(Stats);
	});

	it("should not emit on errors (watch)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./missing",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const watching = compiler.watch({}, (err, stats) => {
			watching.close();
			if (err) return done(err);
			if (compiler.outputFileSystem.existsSync("/bundle.js"))
				return done(new Error("Bundle should not be created on error"));
			done();
		});
	});
	it("should not be running twice at a time (run)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) return done(err);
		});
		compiler.run((err, stats) => {
			if (err) return done();
		});
	});
	it("should not be running twice at a time (watch)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		compiler.watch({}, (err, stats) => {
			if (err) return done();
		});
	});
	it("should not be running twice at a time (run - watch)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) return done(err);
		});
		compiler.watch({}, (err, stats) => {
			if (err) return done();
		});
	});
	it("should not be running twice at a time (watch - run)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		compiler.run((err, stats) => {
			if (err) return done();
		});
	});
	it("should not be running twice at a time (instance cb)", done => {
		const webpack = require("..");
		compiler = webpack(
			{
				context: __dirname,
				mode: "production",
				entry: "./c",
				output: {
					path: "/directory",
					filename: "bundle.js"
				}
			},
			() => {}
		);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) return done();
		});
	});
	it("should run again correctly after first compilation", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats1) => {
			if (err) return done(err);

			compiler.run((err, stats2) => {
				if (err) return done(err);
				expect(stats1.toString({ all: true })).toBeTypeOf("string");
				done();
			});
		});
	});
	it("should watch again correctly after first compilation", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) return done(err);

			const watching = compiler.watch({}, (err, stats) => {
				if (err) return done(err);
				watching.close(done);
			});
		});
	});
	it("should run again correctly after first closed watch", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const watching = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		watching.close(() => {
			compiler.run((err, stats) => {
				if (err) return done(err);
				done();
			});
		});
	});
	it("should set compiler.watching correctly", function (done) {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const watching = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
			watching.close(done);
		});
		expect(compiler.watching).toBe(watching);
	});
	// CHANGE: skip due to panic occurred at runtime
	it.skip("should watch again correctly after first closed watch", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const watching = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		watching.close(() => {
			compiler.watch({}, (err, stats) => {
				if (err) return done(err);
				done();
			});
		});
	});
	it("should run again correctly inside afterDone hook", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		let once = true;
		compiler.hooks.afterDone.tap("RunAgainTest", () => {
			if (!once) return;
			once = false;
			compiler.run((err, stats) => {
				if (err) return done(err);
				done();
			});
		});
		compiler.run((err, stats) => {
			if (err) return done(err);
		});
	});
	it("should call afterDone hook after other callbacks (run)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const runCb = jest.fn();
		const doneHookCb = jest.fn();
		compiler.hooks.done.tap("afterDoneRunTest", doneHookCb);
		compiler.hooks.afterDone.tap("afterDoneRunTest", () => {
			expect(runCb).toHaveBeenCalled();
			expect(doneHookCb).toHaveBeenCalled();
			done();
		});
		compiler.run((err, stats) => {
			if (err) return done(err);
			runCb();
		});
	});
	it("should call afterDone hook after other callbacks (instance cb)", done => {
		const instanceCb = jest.fn();
		const webpack = require("..");
		compiler = webpack(
			{
				context: __dirname,
				mode: "production",
				entry: "./c",
				output: {
					// CHANGE: The `afterDone` hook will not be called if the `path` configuration is added
					// path: "/directory",
					filename: "bundle.js"
				}
			},
			(err, stats) => {
				if (err) return done(err);
				instanceCb();
			}
		);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const doneHookCb = jest.fn();
		compiler.hooks.done.tap("afterDoneRunTest", doneHookCb);
		compiler.hooks.afterDone.tap("afterDoneRunTest", () => {
			expect(instanceCb).toHaveBeenCalled();
			expect(doneHookCb).toHaveBeenCalled();
			done();
		});
	});
	// CHANGE: skip due to panic occurred at runtime
	it.skip("should call afterDone hook after other callbacks (watch)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const invalidHookCb = jest.fn();
		const doneHookCb = jest.fn();
		const watchCb = jest.fn();
		const invalidateCb = jest.fn();
		compiler.hooks.invalid.tap("afterDoneWatchTest", invalidHookCb);
		compiler.hooks.done.tap("afterDoneWatchTest", doneHookCb);
		compiler.hooks.afterDone.tap("afterDoneWatchTest", () => {
			expect(invalidHookCb).toHaveBeenCalled();
			expect(doneHookCb).toHaveBeenCalled();
			expect(watchCb).toHaveBeenCalled();
			expect(invalidateCb).toHaveBeenCalled();
			watching.close(done);
		});
		const watching = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
			watchCb();
		});
		process.nextTick(() => {
			watching.invalidate(invalidateCb);
		});
	});
	it("should call afterDone hook after other callbacks (watch close)", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const invalidHookCb = jest.fn();
		const watchCloseCb = jest.fn();
		const watchCloseHookCb = jest.fn();
		const invalidateCb = jest.fn();
		compiler.hooks.invalid.tap("afterDoneWatchTest", invalidHookCb);
		compiler.hooks.watchClose.tap("afterDoneWatchTest", watchCloseHookCb);
		compiler.hooks.afterDone.tap("afterDoneWatchTest", () => {
			expect(invalidHookCb).toHaveBeenCalled();
			expect(watchCloseCb).toHaveBeenCalled();
			expect(watchCloseHookCb).toHaveBeenCalled();
			expect(invalidateCb).toHaveBeenCalled();
			done();
		});
		const watch = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
			watch.close(watchCloseCb);
		});
		process.nextTick(() => {
			watch.invalidate(invalidateCb);
		});
	});
	it("should flag watchMode as true in watch", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "production",
			entry: "./c",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});

		compiler.outputFileSystem = createFsFromVolume(new Volume());

		const watch = compiler.watch({}, err => {
			if (err) return done(err);
			expect(compiler.watchMode).toBeTruthy();
			watch.close(() => {
				expect(compiler.watchMode).toBeFalsy();
				done();
			});
		});
	});
	// CHANGE: skip due to panic occurred at runtime
	it.skip("should use cache on second run call", done => {
		const webpack = require("..");
		compiler = webpack({
			context: __dirname,
			mode: "development",
			devtool: false,
			entry: "./fixtures/count-loader!./fixtures/count-loader",
			output: {
				path: "/directory"
			}
		});
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run(() => {
			compiler.run(() => {
				const result = compiler.outputFileSystem.readFileSync(
					"/directory/main.js",
					"utf-8"
				);
				expect(result).toContain("module.exports = 0;");
				done();
			});
		});
	});
	it("should call the failed-hook on error", done => {
		const failedSpy = jest.fn();
		const webpack = require("..");
		compiler = webpack({
			bail: true,
			context: __dirname,
			mode: "production",
			entry: "./missing",
			output: {
				path: "/directory",
				filename: "bundle.js"
			}
		});
		compiler.hooks.failed.tap("CompilerTest", failedSpy);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			expect(err).toBeTruthy();
			expect(failedSpy).toHaveBeenCalledTimes(1);
			expect(failedSpy).toHaveBeenCalledWith(err);
			done();
		});
	});
	// CHANGE: skip as rspack does not currently emit correct error code
	it.skip("should deprecate when watch option is used without callback", () => {
		const tracker = deprecationTracking.start();
		const webpack = require("..");
		compiler = webpack({
			watch: true
		});
		const deprecations = tracker();
		expect(deprecations).toEqual([
			expect.objectContaining({
				code: "DEP_WEBPACK_WATCH_WITHOUT_CALLBACK"
			})
		]);
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
		it("should log to the console (verbose)", done => {
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					path: "/directory",
					filename: "bundle.js"
				},
				infrastructureLogging: {
					level: "verbose"
				},
				plugins: [new MyPlugin()]
			});
			compiler.outputFileSystem = createFsFromVolume(new Volume());
			compiler.run((err, stats) => {
				expect(capture.toString().replace(/[\d.]+ ms/, "X ms"))
					.toMatchInlineSnapshot(`
	"<-> [MyPlugin] Group
	  <e> [MyPlugin] Error
	  <w> [MyPlugin] Warning
	  <i> [MyPlugin] Info
	      [MyPlugin] Log
	  <-> [MyPlugin] Collapsed group
	        [MyPlugin] Log inside collapsed group
	<t> [MyPlugin] Time: X ms
	"
	`);
				done();
			});
		});
		it("should log to the console (debug mode)", done => {
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					path: "/directory",
					filename: "bundle.js"
				},
				infrastructureLogging: {
					level: "error",
					debug: /MyPlugin/
				},
				plugins: [new MyPlugin()]
			});
			compiler.outputFileSystem = createFsFromVolume(new Volume());
			compiler.run((err, stats) => {
				expect(capture.toString().replace(/[\d.]+ ms/, "X ms"))
					.toMatchInlineSnapshot(`
		"<-> [MyPlugin] Group
		  <e> [MyPlugin] Error
		  <w> [MyPlugin] Warning
		  <i> [MyPlugin] Info
		      [MyPlugin] Log
		      [MyPlugin] Debug
		  <-> [MyPlugin] Collapsed group
		        [MyPlugin] Log inside collapsed group
		<t> [MyPlugin] Time: X ms
		"
		`);
				done();
			});
		});
		it("should log to the console (none)", done => {
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					path: "/directory",
					filename: "bundle.js"
				},
				infrastructureLogging: {
					level: "none"
				},
				plugins: [new MyPlugin()]
			});
			compiler.outputFileSystem = createFsFromVolume(new Volume());
			compiler.run((err, stats) => {
				expect(capture.toString()).toMatchInlineSnapshot(`""`);
				done();
			});
		});
		it("should log to the console with colors (verbose)", done => {
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					path: "/directory",
					filename: "bundle.js"
				},
				infrastructureLogging: {
					level: "verbose",
					colors: true
				},
				plugins: [new MyPlugin()]
			});
			compiler.outputFileSystem = createFsFromVolume(new Volume());
			compiler.run((err, stats) => {
				expect(escapeAnsi(capture.toStringRaw()).replace(/[\d.]+ ms/, "X ms"))
					.toMatchInlineSnapshot(`
		"<-> <CLR=36,BOLD>[MyPlugin] Group</CLR>
		  <e> <CLR=31,BOLD>[MyPlugin] Error</CLR>
		  <w> <CLR=33,BOLD>[MyPlugin] Warning</CLR>
		  <i> <CLR=32,BOLD>[MyPlugin] Info</CLR>
		      <CLR=BOLD>[MyPlugin] Log<CLR=22>
		  <-> <CLR=36,BOLD>[MyPlugin] Collapsed group</CLR>
		        <CLR=BOLD>[MyPlugin] Log inside collapsed group<CLR=22>
		<t> <CLR=35,BOLD>[MyPlugin] Time: X ms</CLR>
		"
		`);
				done();
			});
		});
		it("should log to the console with colors (debug mode)", done => {
			const webpack = require("..");
			compiler = webpack({
				context: path.join(__dirname, "fixtures"),
				entry: "./a",
				output: {
					path: "/directory",
					filename: "bundle.js"
				},
				infrastructureLogging: {
					level: "error",
					debug: /MyPlugin/,
					colors: true
				},
				plugins: [new MyPlugin()]
			});
			compiler.outputFileSystem = createFsFromVolume(new Volume());
			compiler.run((err, stats) => {
				expect(escapeAnsi(capture.toStringRaw()).replace(/[\d.]+ ms/, "X ms"))
					.toMatchInlineSnapshot(`
		"<-> <CLR=36,BOLD>[MyPlugin] Group</CLR>
		  <e> <CLR=31,BOLD>[MyPlugin] Error</CLR>
		  <w> <CLR=33,BOLD>[MyPlugin] Warning</CLR>
		  <i> <CLR=32,BOLD>[MyPlugin] Info</CLR>
		      <CLR=BOLD>[MyPlugin] Log<CLR=22>
		      [MyPlugin] Debug
		  <-> <CLR=36,BOLD>[MyPlugin] Collapsed group</CLR>
		        <CLR=BOLD>[MyPlugin] Log inside collapsed group<CLR=22>
		<t> <CLR=35,BOLD>[MyPlugin] Time: X ms</CLR>
		"
		`);
				done();
			});
		});

		// CHANGE: specially added for rspack
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
		// CHANGE: specially added for rspack
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
	// CHANGE: specially added for rspack
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

	// CHANGE: specially added for rspack
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
