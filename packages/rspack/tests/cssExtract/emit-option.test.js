/* eslint-env browser */
import fs from "fs";
import path from "path";

import webpack, {
	RspackCssExtractPlugin as MiniCssExtractPlugin
} from "../../src";
import del from "del";

import {
	compile,
	getCompiler,
	getErrors,
	getWarnings,
	runInJsDom
} from "./helpers/index";

describe("emit option", () => {
	it(`should work without emit option`, async () => {
		const compiler = getCompiler(
			"style-url.js",
			{},
			{
				mode: "none",
				output: {
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},

				plugins: [
					new MiniCssExtractPlugin({
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
			"assets"
		);
		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it(`should work when emit option is "true"`, async () => {
		const compiler = getCompiler(
			"style-url.js",
			{
				emit: true
			},
			{
				mode: "none",
				output: {
					path: path.resolve(__dirname, "../outputs")
				},

				plugins: [
					new MiniCssExtractPlugin({
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
			"assets"
		);
		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it(`should work when emit option is "false"`, async () => {
		const compiler = getCompiler(
			"style-url.js",
			{
				emit: false
			},
			{
				mode: "none",
				output: {
					path: path.resolve(__dirname, "../outputs")
				},
				plugins: [
					new MiniCssExtractPlugin({
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
			"assets"
		);
		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it(`should work with locals when emit option is "false"`, async () => {
		const compiler = getCompiler(
			"locals.js",
			{},
			{
				output: {
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},
				module: {
					rules: [
						{
							test: /\.css$/,
							use: [
								{
									loader: MiniCssExtractPlugin.loader,
									options: {
										emit: false
									}
								},
								{
									loader: "css-loader",
									options: {
										modules: true
									}
								}
							]
						}
					]
				},
				plugins: [
					new MiniCssExtractPlugin({
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		runInJsDom("main.bundle.js", compiler, stats, dom => {
			expect(dom.serialize()).toMatchSnapshot("DOM");
		});
		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it(`should work with locals and invalidate cache when emit option is "false"`, async () => {
		const modifyAsset = path.resolve(__dirname, "fixtures", "locals/index.css");
		const modifyAssetContent = fs.readFileSync(modifyAsset);

		class AssetsModifyPlugin {
			constructor(options = {}) {
				this.options = options;
			}

			apply(compiler) {
				compiler.hooks.emit.tapAsync(
					"AssetsModifyPlugin",
					(compilation, callback) => {
						const newContent = modifyAssetContent
							.toString()
							.replace(/foo/i, "foo-bar");
						fs.writeFileSync(this.options.file, newContent);

						callback();
					}
				);
			}
		}

		const outputPath = path.resolve(__dirname, "./js/cache-memory");
		const webpackConfig = {
			mode: "development",
			context: path.resolve(__dirname, "./fixtures"),
			cache: true,
			entry: "./locals.js",
			output: {
				path: outputPath
			},
			module: {
				rules: [
					{
						test: /\.css$/,
						use: [
							{
								loader: MiniCssExtractPlugin.loader,
								options: {
									emit: false
								}
							},
							{
								loader: "css-loader",
								options: {
									modules: true
								}
							}
						]
					}
				]
			},
			plugins: [
				new MiniCssExtractPlugin({
					filename: "[name].css"
				}),
				new AssetsModifyPlugin({
					file: modifyAsset
				})
			],
			experiments: {
				css: false,
				rspackFuture: {
					newTreeshaking: true
				}
			}
		};

		await del([outputPath]);

		const compiler1 = webpack(webpackConfig);

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
						`assets`
					);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchSnapshot(`emittedAssets`);
					runInJsDom("main.js", compiler1, stats, dom => {
						expect(dom.serialize()).toMatchSnapshot("DOM");
					});
					expect(getWarnings(stats)).toMatchSnapshot("warnings");
					expect(getErrors(stats)).toMatchSnapshot("errors");

					resolve();
				});
			});
		});

		const compiler2 = webpack(webpackConfig);

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
						`assets`
					);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchSnapshot(`emittedAssets`);
					runInJsDom("main.js", compiler2, stats, dom => {
						expect(dom.serialize()).toMatchSnapshot("DOM");
					});
					expect(getWarnings(stats)).toMatchSnapshot("warnings");
					expect(getErrors(stats)).toMatchSnapshot("errors");

					resolve();
				});
			});
		});

		fs.writeFileSync(modifyAsset, modifyAssetContent);
	});

	it('should work with the "memory" cache and disabled "emit" option', async () => {
		const outputPath = path.resolve(__dirname, "./js/cache-memory");
		const webpackConfig = {
			mode: "development",
			context: path.resolve(__dirname, "fixtures"),
			cache: true,
			output: {
				path: outputPath
			},
			entry: "./style-url.js",
			module: {
				rules: [
					{
						test: /\.css$/,
						use: [
							{
								loader: MiniCssExtractPlugin.loader,
								options: {
									emit: false
								}
							},
							"css-loader"
						]
					},
					{
						test: /\.svg$/,
						type: "asset/resource",
						generator: {
							filename: "static/[name][ext][query]"
						}
					}
				]
			},
			plugins: [
				new MiniCssExtractPlugin({
					filename: "[name].css"
				})
			],
			experiments: {
				css: false,
				rspackFuture: {
					newTreeshaking: true
				}
			}
		};

		await del([outputPath]);

		const compiler1 = webpack(webpackConfig);

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
						`assets`
					);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchSnapshot(`emittedAssets`);
					expect(getWarnings(stats)).toMatchSnapshot("warnings");
					expect(getErrors(stats)).toMatchSnapshot("errors");

					resolve();
				});
			});
		});

		const compiler2 = webpack(webpackConfig);

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
						`assets`
					);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchSnapshot(`emittedAssets`);
					expect(getWarnings(stats)).toMatchSnapshot("warnings");
					expect(getErrors(stats)).toMatchSnapshot("errors");

					resolve();
				});
			});
		});
	});

	it('should invalidate the cache with disabled "emit" option', async () => {
		class AssetsModifyPlugin {
			constructor(options = {}) {
				this.options = options;
			}

			apply(compiler) {
				compiler.hooks.emit.tapAsync(
					"AssetsModifyPlugin",
					(compilation, callback) => {
						fs.writeFileSync(this.options.file, `.a{color: red;}`);

						callback();
					}
				);
			}
		}

		const outputPath = path.resolve(__dirname, "./js/cache-memory");
		const modifyAsset = path.resolve(__dirname, "fixtures", "style-url.css");
		const modifyAssetContent = fs.readFileSync(modifyAsset);
		const webpackConfig = {
			mode: "development",
			context: path.resolve(__dirname, "fixtures"),
			cache: true,
			output: {
				path: outputPath
			},
			entry: "./style-url.js",
			module: {
				rules: [
					{
						test: /\.css$/,
						use: [
							{
								loader: MiniCssExtractPlugin.loader,
								options: {
									emit: false
								}
							},
							"css-loader"
						]
					},
					{
						test: /\.svg$/,
						type: "asset/resource",
						generator: {
							filename: "static/[name][ext][query]"
						}
					}
				]
			},
			plugins: [
				new MiniCssExtractPlugin({
					filename: "[name].css"
				}),
				new AssetsModifyPlugin({
					file: modifyAsset
				})
			],
			experiments: {
				css: false,
				rspackFuture: {
					newTreeshaking: true
				}
			}
		};

		await del([outputPath]);

		const compiler1 = webpack(webpackConfig);

		await new Promise((resolve, reject) => {
			compiler1.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler1.close(() => {
					expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
						`assets`
					);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchSnapshot(`emittedAssets`);
					expect(getWarnings(stats)).toMatchSnapshot("warnings");
					expect(getErrors(stats)).toMatchSnapshot("errors");

					resolve();
				});
			});
		});

		const compiler2 = webpack(webpackConfig);

		await new Promise((resolve, reject) => {
			compiler2.run((error, stats) => {
				if (error) {
					reject(error);

					return;
				}

				compiler2.close(() => {
					expect(Object.keys(stats.compilation.assets).sort()).toMatchSnapshot(
						`assets`
					);
					// expect(
					// 	Array.from(stats.compilation.emittedAssets).sort()
					// ).toMatchSnapshot(`emittedAssets`);
					expect(getWarnings(stats)).toMatchSnapshot("warnings");
					expect(getErrors(stats)).toMatchSnapshot("errors");

					resolve();
				});
			});
		});

		fs.writeFileSync(modifyAsset, modifyAssetContent);
	});
});
