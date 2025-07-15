"use strict";

require("../../dist/helper/legacy/warmup-webpack");
const path = require("path");
const webpack = require("@rspack/core");
const { LazyCompilationTestPlugin } = require("@rspack/test-tools");

describe("MultiCompiler", function () {
	jest.setTimeout(20000);

	// fix issue #2585
	it("should respect parallelism when using watching", done => {
		const configMaps = [];

		for (let index = 0; index < 3; index++) {
			configMaps.push({
				name: index.toString(),
				mode: "development",
				entry: "./src/main.jsx",
				devServer: {
					hot: true
				}
			});
		}
		configMaps.parallelism = 1;
		const compiler = webpack(configMaps);

		compiler.watch({}, err => {
			if (err) {
				compiler.close(() => {
					done(err);
				});
				return;
			}
			compiler.close(err => {
				if (err) return done(err);
				done();
			});
		});
	}, 20000);
});

describe("with lazy compilation", function () {
	it("compiler has unique lazy compilation config", done => {
		const context = path.join(__dirname, "../fixtures");
		const configs = [
			{
				entry: "./esm/a.js",
				context
			},
			{
				entry: "./esm/b.js",
				experiments: {
					lazyCompilation: {
						entries: true
					}
				},
				context
			},
			{
				entry: "./esm/d.js",
				experiments: {
					lazyCompilation: {
						entries: false,
						imports: true
					}
				},
				context
			}
		];
		const compiler = webpack(configs);

		new LazyCompilationTestPlugin().apply(compiler);

		compiler.watch({}, (err, multiStats) => {
			if (err) {
				compiler.close(() => {
					done(err);
				});
				return;
			}

			const [statsA, statsB, statsC] = multiStats.stats;

			expect(
				statsA.toJson().modules.every(module => {
					return !module.identifier.includes("lazy-compilation-proxy");
				})
			).toBeTruthy();

			// second compiler lazy compile entry
			expect(
				statsB.toJson().modules.find(module => {
					return (
						module.identifier.includes("lazy-compilation-proxy") &&
						module.identifier.replaceAll("\\", "/").includes("/esm/b.js")
					);
				})
			).toBeDefined();

			// third compiler lazy compile dyn imports
			expect(
				statsC.toJson().modules.find(module => {
					return (
						module.identifier.includes("lazy-compilation-proxy") &&
						module.identifier
							.replaceAll("\\", "/")
							.includes("/esm/d-dynamic.js")
					);
				})
			).toBeDefined();

			compiler.close(err => {
				if (err) return done(err);
				done();
			});
		});
	});
});

describe.skip("Pressure test", function () {
	it("should work well in multiCompilers", done => {
		const configs = Array(100).fill({
			context: path.join(__dirname, "../fixtures"),
			entry: "./a.js"
		});

		const multiCompiler = webpack(configs);

		multiCompiler.run(err => {
			if (err) done(err);
			else done();
		});
	});

	it("should work well in concurrent", async () => {
		const total = 100;

		let finish = 0;

		const runnings = [];

		for (let i = 0; i < total; i++) {
			if (i % 10 == 0) {
				// Insert new instance while we are running
				webpack(
					{
						context: path.join(__dirname, "fixtures"),
						entry: "./a.js"
					},
					() => {}
				);
			}

			runnings.push(
				new Promise(resolve => {
					webpack(
						{
							context: path.join(__dirname, "fixtures"),
							entry: "./a.js"
						},
						err => {
							resolve(null);
							if (!err) finish++;
						}
					);
				})
			);
		}

		await Promise.all(runnings);
		expect(finish).toBe(total);
	});
});
