/**
 * @jest-environment node
 */

import path from "path";

import webpack from "../../";
import del from "del";

describe("TestCache", () => {
	afterEach(() => {
		jest.clearAllMocks();
	});

	it("should work without cache", async () => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "asset-modules");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const outputPath = path.resolve(__dirname, "js/cache-false");

		await del([outputPath]);

		const compiler1 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: false,
			output: {
				path: outputPath
			},
			experiments: {
				css: false,
				rspackFuture: {
					disableApplyEntryLazily: true,
					newTreeshaking: true
				}
			}
		});

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				[
				  "main.css",
				  "main.js",
				  "static/react.svg",
				]
			`);
					// expect(Array.from(stats.compilation.emittedAssets).sort())
					// 	.toMatchInlineSnapshot(`
					//   Array [
					//     "main.css",
					//     "main.js",
					//     "static/react.svg",
					//   ]
					// `);
					expect([...stats.compilation.warnings]).toHaveLength(0);
					expect([...stats.compilation.errors]).toHaveLength(0);

					resolve();
				});
			});
		});

		const compiler2 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: false,
			output: {
				path: outputPath
			},
			experiments: { css: false, rspackFuture: { newTreeshaking: true } }
		});

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				[
				  "main.css",
				  "main.js",
				  "static/react.svg",
				]
			`);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchInlineSnapshot(`Array []`);
					expect([...stats.compilation.warnings]).toHaveLength(0);
					expect([...stats.compilation.errors]).toHaveLength(0);

					resolve();
				});
			});
		});
	});

	it('should work with the "memory" cache', async () => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "asset-modules");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const outputPath = path.resolve(__dirname, "js/cache-memory");

		await del([outputPath]);

		const compiler1 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: true,
			output: {
				path: outputPath
			},
			experiments: { css: false, rspackFuture: { newTreeshaking: true } }
		});

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				[
				  "main.css",
				  "main.js",
				  "static/react.svg",
				]
			`);
					// expect(Array.from(stats.compilation.emittedAssets).sort())
					// 	.toMatchInlineSnapshot(`
					//   Array [
					//     "main.css",
					//     "main.js",
					//     "static/react.svg",
					//   ]
					// `);
					expect([...stats.compilation.warnings]).toHaveLength(0);
					expect([...stats.compilation.errors]).toHaveLength(0);

					resolve();
				});
			});
		});

		const compiler2 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: true,
			output: {
				path: outputPath
			},
			experiments: {
				css: false,
				rspackFuture: {
					disableApplyEntryLazily: true,
					newTreeshaking: true
				}
			}
		});

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				[
				  "main.css",
				  "main.js",
				  "static/react.svg",
				]
			`);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchInlineSnapshot(`Array []`);
					expect([...stats.compilation.warnings]).toHaveLength(0);
					expect([...stats.compilation.errors]).toHaveLength(0);

					resolve();
				});
			});
		});
	});

	it.skip('should work with the "filesystem" cache', async () => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "simple");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const outputPath = path.resolve(__dirname, "js/cache-filesystem");
		const fileSystemCacheDirectory = path.resolve(
			__dirname,
			"./js/.cache/type-filesystem"
		);

		await del([outputPath, fileSystemCacheDirectory]);

		const compiler1 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			},
			experiments: {
				css: false,
				rspackFuture: { newTreeshaking: true }
			}
		});

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				            ]
			          `);
					// expect(Array.from(stats.compilation.emittedAssets).sort())
					// 	.toMatchInlineSnapshot(`
					//   Array [
					//     "main.css",
					//     "main.js",
					//   ]
					// `);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});

		const compiler2 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				            ]
			          `);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchInlineSnapshot(`Array []`);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});
	});

	it.skip('should work with the "filesystem" cache #2', async () => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "at-import-layer");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const outputPath = path.resolve(__dirname, "js/cache-filesystem-1");
		const fileSystemCacheDirectory = path.resolve(
			__dirname,
			"./js/.cache/type-filesystem-1"
		);

		await del([outputPath, fileSystemCacheDirectory]);

		const compiler1 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				            ]
			          `);
					// expect(Array.from(stats.compilation.emittedAssets).sort())
					// 	.toMatchInlineSnapshot(`
					//   Array [
					//     "main.css",
					//     "main.js",
					//   ]
					// `);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});

		const compiler2 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				            ]
			          `);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchInlineSnapshot(`Array []`);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});
	});

	it.skip('should work with the "filesystem" cache and asset modules', async () => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "asset-modules");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const outputPath = path.resolve(
			__dirname,
			"js/cache-filesystem-asset-modules"
		);
		const fileSystemCacheDirectory = path.resolve(
			__dirname,
			"./js/.cache/type-filesystem"
		);

		await del([outputPath, fileSystemCacheDirectory]);

		const compiler1 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				              "static/react.svg",
				            ]
			          `);
					// expect(Array.from(stats.compilation.emittedAssets).sort())
					// 	.toMatchInlineSnapshot(`
					//   Array [
					//     "main.css",
					//     "main.js",
					//     "static/react.svg",
					//   ]
					// `);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});

		const compiler2 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				              "static/react.svg",
				            ]
			          `);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchInlineSnapshot(`Array []`);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});
	});

	it.skip('should work with the "filesystem" cache and file-loader', async () => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "file-loader");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const outputPath = path.resolve(
			__dirname,
			"js/cache-filesystem-file-loader"
		);
		const fileSystemCacheDirectory = path.resolve(
			__dirname,
			"./js/.cache/type-filesystem"
		);

		await del([outputPath, fileSystemCacheDirectory]);

		const compiler1 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				              "static/react.svg",
				            ]
			          `);
					// expect(Array.from(stats.compilation.emittedAssets).sort())
					// 	.toMatchInlineSnapshot(`
					//   Array [
					//     "main.css",
					//     "main.js",
					//     "static/react.svg",
					//   ]
					// `);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});

		const compiler2 = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: {
				type: "filesystem",
				cacheDirectory: fileSystemCacheDirectory,
				idleTimeout: 0,
				idleTimeoutForInitialStore: 0
			},
			output: {
				path: outputPath
			}
		});

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort())
						.toMatchInlineSnapshot(`
				            Array [
				              "main.css",
				              "main.js",
				              "static/react.svg",
				            ]
			          `);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchInlineSnapshot(`Array []`);
					expect(stats.compilation.warnings).toHaveLength(0);
					expect(stats.compilation.errors).toHaveLength(0);

					resolve();
				});
			});
		});
	});
});
