"use strict";

const path = require("path");
const fs = require("fs");
const { rspack } = require("@rspack/core");
const rimraf = require("rimraf");
const _ = require("lodash");
const { default: HtmlWebpackPlugin, defaultTemplateCompiler } = require("../");

const OUTPUT_DIR = path.resolve(__dirname, "./dist/basic-spec");

process.on("unhandledRejection", r => console.log(r));

const pugTemplateCompiler = {
	async compile(content, options) {
		const pug = require("pug");
		return `${pug.compileClient(content, {
			filename: options.filename,
			doctype: "html",
			compileDebug: false
		})};\ntemplate;`;
	}
};

const htmlLoaderTemplateCompiler = {
	async compile(content, { filename }) {
		const { runLoaders } = require("loader-runner");
		let [loaderReturnCode] = await new Promise(resolve => {
			runLoaders(
				{
					resource: filename,
					loaders: [require.resolve("html-loader")],
					context: {
						getOptions() {
							return { esModule: false };
						}
					},
					readResource: fs.readFile.bind(fs)
				},
				(err, out) => resolve(out.result)
			);
		});
		return `var module = {};\n${loaderReturnCode};\nmodule.exports`;
	}
};

function testHtmlPlugin(
	webpackConfig,
	expectedResults,
	outputFile,
	done,
	expectErrors,
	expectWarnings
) {
	outputFile = outputFile || "index.html";
	const envSetup = () => {
		let prevEnv = process.env.NODE_ENV;
		process.env.NODE_ENV = webpackConfig.mode;
		return () => {
			process.env.NODE_ENV = prevEnv;
		};
	};

	let restoreEnv = envSetup();
	let prevDone = done;
	done = (...args) => {
		restoreEnv();
		prevDone(...args);
	};

	rspack({ ...webpackConfig, builtins: { minify: false } }, (err, stats) => {
		expect(err).toBeFalsy();
		const statsJson = stats.toJson({ all: true });
		// console.log(statsJson.chunks);
		// console.log(statsJson.modules);
		const compilationErrors = statsJson.errors.map(e => e.message).join("\n");
		if (expectErrors) {
			expect(compilationErrors).not.toBe("");
		} else {
			expect(compilationErrors).toBe("");
		}
		const compilationWarnings = statsJson.warnings
			.map(e => e.message)
			.join("\n");
		if (expectWarnings) {
			expect(compilationWarnings).not.toBe("");
		} else {
			expect(compilationWarnings).toBe("");
		}
		if (outputFile instanceof RegExp) {
			const fileNames = statsJson.assets.map(a => a.name);
			const matches = statsJson.assets
				.map(a => a.name)
				.filter(item => outputFile.test(item));
			expect(matches[0] || fileNames).not.toEqual(fileNames);
			outputFile = matches[0];
		}
		expect(outputFile.indexOf("[hash]") === -1).toBe(true);
		const outputFileExists = fs.existsSync(path.join(OUTPUT_DIR, outputFile));
		expect(outputFileExists).toBe(true);
		if (!outputFileExists) {
			return done();
		}
		const htmlContent = fs
			.readFileSync(path.join(OUTPUT_DIR, outputFile))
			.toString();
		let chunksInfo;
		for (let i = 0; i < expectedResults.length; i++) {
			const expectedResult = expectedResults[i];
			try {
				if (expectedResult instanceof RegExp) {
					expect(htmlContent).toMatch(expectedResult);
				} else if (typeof expectedResult === "object") {
					if (expectedResult.type === "chunkhash") {
						if (!chunksInfo) {
							chunksInfo = getChunksInfoFromStats(stats);
						}
						// TODO: stats.hash
						// const chunkhash = chunksInfo[expectedResult.chunkName].hash;
						expect(htmlContent).toContain(
							// expectedResult.containStr.replace("%chunkhash%", chunkhash)
							expectedResult
						);
					}
				} else {
					expect(htmlContent).toContain(
						// TODO: stats.hash
						// expectedResult.replace("%hash%", stats.hash)
						expectedResult
					);
				}
			} catch (e) {
				done(e);
				return;
			}
		}
		done();
	});
}

function getChunksInfoFromStats(stats) {
	const chunks = stats.toJson().chunks;
	const chunksInfo = {};
	for (let i = 0; i < chunks.length; i++) {
		const chunk = chunks[i];
		const chunkName = chunk.names[0];
		if (chunkName) {
			chunksInfo[chunkName] = chunk;
		}
	}
	return chunksInfo;
}

describe("HtmlWebpackPlugin", () => {
	jest.setTimeout(process.env.CI ? 120000 : 10000);

	beforeEach(done => {
		rimraf(OUTPUT_DIR, done);
	});

	it("generates a default index.html file for a single entry point", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[/<script defer="defer" src="main.js"><\/script>[\s]*<\/head>/],
			null,
			done
		);
	});

	it("properly encodes file names in emitted URIs", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					"foo/very fancy+name": path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[
				/<script defer="defer" src="foo\/very%20fancy%2Bname.js"><\/script>[\s]*<\/head>/
			],
			null,
			done
		);
	});

	(process.platform === "win32" || process.platform === "win64"
		? it.skip
		: it.skip)(
		"properly encodes file names in emitted URIs but keeps the querystring",
		done => {
			testHtmlPlugin(
				{
					mode: "production",
					entry: path.join(__dirname, "fixtures/index.js"),
					output: {
						path: OUTPUT_DIR,
						filename:
							"fo:o/very fancy+file-name.js?path=/home?value=abc&value=def#zzz"
					},
					plugins: [new HtmlWebpackPlugin()]
				},
				[
					'<script defer="defer" src="fo%3Ao/very%20fancy%2Bfile-name.js?path=/home?value=abc&value=def#zzz">'
				],
				null,
				done
			);
		}
	);

	it("generates a default index.html file with multiple entry points", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[
				'<script defer="defer" src="util_bundle.js"',
				'<script defer="defer" src="app_bundle.js"'
			],
			null,
			done
		);
	});

	it("allows you to specify a custom template compiler", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: false,
						template: path.join(__dirname, "fixtures/template.pug"),
						templateCompiler: pugTemplateCompiler
					})
				]
			},
			['<script src="app_bundle.js', "Some unique text"],
			null,
			done
		);
	});

	it.skip("should pass through template compiler errors", done => {
		testHtmlPlugin(
			{
				mode: "production",
				optimization: {
					emitOnErrors: true
				},
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: false,
						template: path.join(__dirname, "fixtures/invalid.html")
					})
				]
			},
			["ReferenceError: foo is not defined"],
			null,
			done,
			true
		);
	});

	it("works when using html-loader as template compiler", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						template: path.join(__dirname, "fixtures/plain.html"),
						templateCompiler: htmlLoaderTemplateCompiler
					})
				]
			},
			['<script defer="defer" src="app_bundle.js"'],
			null,
			done
		);
	});

	it("allows you to specify your own HTML template file", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/test.html"),
						inject: false
					})
				]
			},
			['<script src="app_bundle.js', "Some unique text"],
			null,
			done
		);
	});

	it("allows to use a function to map entry names to filenames", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: entry => `${entry}.html`
					})
				]
			},
			['<script defer="defer" src="app_bundle.js'],
			"app.html",
			done
		);
	});

	it("allows to use [name] for file names", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: "[name].html"
					})
				]
			},
			['<script defer="defer" src="app_bundle.js'],
			"app.html",
			done
		);
	});

	it("picks up src/index.ejs by default", done => {
		testHtmlPlugin(
			{
				mode: "production",
				context: path.join(__dirname, "fixtures"),
				entry: {
					app: "./index.js"
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="app_bundle.js', "src/index.ejs"],
			null,
			done
		);
	});

	it("allows you to inject the assets into a given html file", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						template: path.join(__dirname, "fixtures/plain.html")
					})
				]
			},
			[
				'<script defer="defer" src="util_bundle.js"',
				'<script defer="defer" src="app_bundle.js"'
			],
			null,
			done
		);
	});

	it("allows you to inject the assets into the body of the given template", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: "body",
						template: path.join(__dirname, "fixtures/plain.html")
					})
				]
			},
			[
				'<script defer="defer" src="util_bundle.js"',
				'<script defer="defer" src="app_bundle.js"'
			],
			null,
			done
		);
	});

	it("allows you to inject the assets into the head of the given template", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: "head",
						template: path.join(__dirname, "fixtures/plain.html")
					})
				]
			},
			[
				'<script defer="defer" src="util_bundle.js"',
				'<script defer="defer" src="app_bundle.js"'
			],
			null,
			done
		);
	});

	it("allows you to inject a specified asset into a given html file", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						chunks: ["app"],
						template: path.join(__dirname, "fixtures/plain.html")
					})
				]
			},
			['<script defer="defer" src="app_bundle.js"'],
			null,
			done
		);
	});

	it("allows you to inject a specified asset into a given html file", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						excludeChunks: ["util"],
						template: path.join(__dirname, "fixtures/plain.html")
					})
				]
			},
			['<script defer="defer" src="app_bundle.js"'],
			null,
			done
		);
	});

	it.skip("allows you to use chunkhash with asset into a given html file", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: false,
						template: path.join(__dirname, "fixtures/webpackconfig.html")
					})
				]
			},
			[
				{
					type: "chunkhash",
					chunkName: "app",
					containStr: '<script src="app_bundle.js"'
				}
			],
			null,
			done
		);
	});

	it("allows you to disable injection", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: false,
						template: path.join(__dirname, "fixtures/plain.html")
					})
				]
			},
			["<body></body>"],
			null,
			done
		);
	});

	it("allows you to specify your own HTML template function", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: { app: path.join(__dirname, "fixtures/index.js") },
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						templateContent: function () {
							return fs.readFileSync(
								path.join(__dirname, "fixtures/plain.html"),
								"utf8"
							);
						}
					})
				]
			},
			['<script defer="defer" src="app.js"'],
			null,
			done
		);
	});

	it("works with source maps", done => {
		testHtmlPlugin(
			{
				mode: "development",
				devtool: "source-map",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer src="main_bundle.js"'],
			null,
			done
		);
	});

	it("works with process (issue#2179)", done => {
		testHtmlPlugin(
			{
				mode: "development",
				devtool: "source-map",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/issue2179.html")
					})
				]
			},
			["console.log(1)"],
			null,
			done
		);
	});

	it.skip("handles hashes in bundle filenames", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle_[hash].js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[/<script defer="defer" src="index_bundle_[0-9a-f]+\.js"*/],
			null,
			done
		);
	});

	it.skip("handles hashes in the directory which has the bundle file", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/dist/[hash]/",
					filename: "index_bundle_[hash].js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[
				/<script defer="defer" src="\/dist\/[0-9a-f]+\/index_bundle_[0-9a-f]+\.js"*/
			],
			null,
			done
		);
	});

	it.skip("allows to append hashes to the assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [new HtmlWebpackPlugin({ hash: true })]
			},
			['<script defer="defer" src="index_bundle.js?%hash%"'],
			null,
			done
		);
	});

	it.skip("allows to append hashes to the assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [new HtmlWebpackPlugin({ hash: true, inject: true })]
			},
			['<script defer="defer" src="index_bundle.js?%hash%"'],
			null,
			done
		);
	});

	it("should work with the css", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<link href="main.css" rel="stylesheet">'],
			null,
			done
		);
	});

	it("should work with the css on windows and protocol relative urls support (#205)", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js",
					publicPath: "//localhost:8080/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<link href="//localhost:8080/main.css"'],
			null,
			done
		);
	});

	it.skip("should allow to add cache hashes to with the css assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js",
					publicPath: "/some/"
				},
				module: {
					rules: [
						{ test: /\.css$/, use: [MiniCssExtractPlugin.loader, "css-loader"] }
					]
				},
				plugins: [
					new HtmlWebpackPlugin({
						hash: true,
						filename: path.resolve(OUTPUT_DIR, "subfolder", "test.html")
					}),
					new MiniCssExtractPlugin({ filename: "styles.css" })
				]
			},
			['<link href="/some/styles.css?%hash%"'],
			path.join("subfolder", "test.html"),
			done
		);
	});

	it.skip("should allow to add cache hashes to with the css assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js",
					publicPath: "/some"
				},
				module: {
					rules: [
						{ test: /\.css$/, use: [MiniCssExtractPlugin.loader, "css-loader"] }
					]
				},
				plugins: [
					new HtmlWebpackPlugin({ hash: true }),
					new MiniCssExtractPlugin({ filename: "styles.css" })
				]
			},
			['<link href="/some/styles.css?%hash%"'],
			null,
			done
		);
	});

	it.skip("should allow to add cache hashes to with the css assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js",
					publicPath: "some/"
				},
				module: {
					rules: [
						{ test: /\.css$/, use: [MiniCssExtractPlugin.loader, "css-loader"] }
					]
				},
				plugins: [
					new HtmlWebpackPlugin({ hash: true }),
					new MiniCssExtractPlugin({ filename: "styles.css" })
				]
			},
			['<link href="some/styles.css?%hash%"'],
			null,
			done
		);
	});

	it.skip("should allow to add cache hashes to with the css assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				module: {
					rules: [
						{ test: /\.css$/, use: [MiniCssExtractPlugin.loader, "css-loader"] }
					]
				},
				plugins: [
					new HtmlWebpackPlugin({ hash: true }),
					new MiniCssExtractPlugin({ filename: "styles.css" })
				]
			},
			['<link href="styles.css?%hash%"'],
			null,
			done
		);
	});

	it.skip("should allow to add cache hashes to with the css assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				module: {
					rules: [
						{ test: /\.css$/, use: [MiniCssExtractPlugin.loader, "css-loader"] }
					]
				},
				plugins: [
					new HtmlWebpackPlugin({
						hash: true,
						filename: path.resolve(OUTPUT_DIR, "subfolder", "test.html")
					}),
					new MiniCssExtractPlugin({ filename: "styles.css" })
				]
			},
			['<link href="../styles.css?%hash%"'],
			path.join("subfolder", "test.html"),
			done
		);
	});

	it("should inject css files when using with css", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin({ inject: true })]
			},
			['<link href="main.css"'],
			null,
			done
		);
	});

	it.skip("should allow to add cache hashes to with injected css assets", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				module: {
					rules: [
						{ test: /\.css$/, use: [MiniCssExtractPlugin.loader, "css-loader"] }
					]
				},
				plugins: [
					new HtmlWebpackPlugin({ hash: true, inject: true }),
					new MiniCssExtractPlugin({ filename: "styles.css" })
				]
			},
			['<link href="styles.css?%hash%"'],
			null,
			done
		);
	});

	it("should output xhtml link stylesheet tag", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						xhtml: true,
						minify: {
							keepClosingSlash: true
						}
					})
				]
			},
			['<link href="main.css" rel="stylesheet"/>'],
			null,
			done
		);
	});

	it("prepends the publicPath to function", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="/main.js"'],
			null,
			done
		);
	});

	it("prepends the publicPath to /some/", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/some/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="/some/main.js"'],
			null,
			done
		);
	});

	it("prepends the publicPath to /some", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/some"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="/some/main.js"'],
			null,
			done
		);
	});

	it("prepends the publicPath to /some", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "some/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="some/main.js"'],
			null,
			done
		);
	});

	it("prepends the publicPath to undefined", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="main.js"'],
			null,
			done
		);
	});

	it("prepends the publicPath to undefined", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: path.resolve(OUTPUT_DIR, "subfolder", "test.html")
					})
				]
			},
			['<script defer="defer" src="../main.js"'],
			path.join("subfolder", "test.html"),
			done
		);
	});

	it('prepends the publicPath to script defer="defer" src', done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "http://cdn.example.com/assets/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="http://cdn.example.com/assets/main.js"'],
			null,
			done
		);
	});

	it("handles subdirectories in the webpack output bundles", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js",
					publicPath: "/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="/assets/main.js"'],
			null,
			done
		);
	});

	it("allows to set public path to an empty string", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js",
					publicPath: ""
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: "foo/index.html"
					})
				]
			},
			['<script defer="defer" src="assets/main.js"'],
			"foo/index.html",
			done
		);
	});

	it("allows to set the html-webpack-plugin public path to an empty string", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js",
					publicPath: "/"
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: "foo/index.html",
						publicPath: ""
					})
				]
			},
			['<script defer="defer" src="assets/main.js"'],
			"foo/index.html",
			done
		);
	});

	it("handles subdirectories in the webpack output bundles along with a relative path", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="assets/main.js"'],
			null,
			done
		);
	});

	it("handles subdirectories in the webpack output bundles along with a relative path", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: path.resolve(OUTPUT_DIR, "subfolder", "test.html")
					})
				]
			},
			['<script defer="defer" src="../assets/main.js"'],
			path.join("subfolder", "test.html"),
			done
		);
	});

	it("handles subdirectories in the webpack output bundles along with a absolute path", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js",
					publicPath: "http://cdn.example.com/"
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			['<script defer="defer" src="http://cdn.example.com/assets/main.js"'],
			null,
			done
		);
	});

	it("allows you to configure the title of the generated HTML page", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ title: "My Cool App" })]
			},
			["<title>My Cool App</title>"],
			null,
			done
		);
	});

	it("allows you to configure the output filename", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ filename: "test.html" })]
			},
			['<script defer="defer" src="main.js"'],
			"test.html",
			done
		);
	});

	it.skip("will replace [hash] in the filename with the child compilation hash", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: "test-[hash].html"
					})
				]
			},
			['<script defer="defer" src="index_bundle.js"'],
			/test-\S+\.html$/,
			done
		);
	});

	it.skip("should work with hash options provided in output options", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					index: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js",
					hashDigestLength: 16
				},
				plugins: [
					new HtmlWebpackPlugin({ filename: "index.[contenthash].html" })
				]
			},
			[],
			/index\.[a-z0-9]{16}\.html/,
			done
		);
	});

	it.skip("should allow filename in the format of [contenthash:<length>]", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					index: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({ filename: "index.[contenthash:4].html" })
				]
			},
			[],
			/index\.[a-z0-9]{4}\.html/,
			done
		);
	});

	it.skip("will replace [contenthash] in the filename with a content hash of 32 hex characters", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					index: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({ filename: "index.[contenthash].html" })
				]
			},
			[],
			/index\.[a-f0-9]{20}\.html/,
			done
		);
	});

	it("allows you to use an absolute output filename", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: path.resolve(OUTPUT_DIR, "subfolder", "test.html")
					})
				]
			},
			['<script defer="defer" src="../main.js"'],
			path.join("subfolder", "test.html"),
			done
		);
	});

	it("allows you to use an absolute output filename outside the output path", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: path.join(OUTPUT_DIR, "app")
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: path.resolve(OUTPUT_DIR, "test.html")
					})
				]
			},
			['<script defer="defer" src="app/main.js"'],
			"test.html",
			done
		);
	});

	it("allows you to use an relative output filename outside the output path", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: path.join(OUTPUT_DIR, "app")
				},
				plugins: [
					new HtmlWebpackPlugin({
						filename: "../test.html"
					})
				]
			},
			['<script defer="defer" src="app/main.js"'],
			"test.html",
			done
		);
	});

	it("will try to use a relative name if the filename is in a subdirectory", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ filename: "assets/test.html" })]
			},
			['<script defer="defer" src="../main.js"'],
			"assets/test.html",
			done
		);
	});

	it('will try to use a relative name if the filename and the script defer="defer" are in a subdirectory', done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "assets/[name].js"
				},
				plugins: [new HtmlWebpackPlugin({ filename: "assets/demo/test.html" })]
			},
			['<script defer="defer" src="../../assets/main.js"'],
			"assets/demo/test.html",
			done
		);
	});

	it("allows you write multiple HTML files", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin(),
					new HtmlWebpackPlugin({
						filename: "second-file.html",
						template: path.join(__dirname, "fixtures/test.html")
					}),
					new HtmlWebpackPlugin({
						filename: "third-file.html",
						template: path.join(__dirname, "fixtures/test.html")
					})
				]
			},
			['<script defer="defer" src="app.js"'],
			null,
			() => {
				expect(fs.existsSync(path.join(OUTPUT_DIR, "second-file.html"))).toBe(
					true
				);
				expect(fs.existsSync(path.join(OUTPUT_DIR, "third-file.html"))).toBe(
					true
				);
				done();
			}
		);
	});

	it("should inject js css files even if the html file is incomplete", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/empty_html.html")
					})
				]
			},
			['<link href="main.css"', '<script defer="defer" src="main.js"'],
			null,
			done
		);
	});

	it("exposes the webpack configuration to templates", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					publicPath: "https://cdn.com",
					filename: "[name].js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/webpackconfig.html")
					})
				]
			},
			["Public path is https://cdn.com"],
			null,
			done
		);
	});

	it("fires the html-webpack-plugin-alter-asset-tags event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).alterAssetTags.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							expect(Object.keys(object.assetTags)).toEqual([
								"scripts",
								"styles",
								"meta"
							]);
							eventFired = true;
							callback();
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("allows events to add a no-value attribute", done => {
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).alterAssetTags.tapAsync(
						"HtmlWebpackPluginTest",
						(pluginArgs, callback) => {
							pluginArgs.assetTags.scripts = pluginArgs.assetTags.scripts.map(
								tag => {
									if (tag.tagName === "script") {
										tag.attributes.specialAttribute = true;
									}
									return tag;
								}
							);
							callback(null, pluginArgs);
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[
				/[\s]*<script defer="defer" src="app.js" specialattribute><\/script>[\s]*<\/head>/
			],
			null,
			done,
			false,
			false
		);
	});

	it("allows events to remove an attribute by setting it to false", done => {
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).alterAssetTags.tapAsync(
						"HtmlWebpackPluginTest",
						(pluginArgs, callback) => {
							pluginArgs.assetTags.scripts = pluginArgs.assetTags.scripts.map(
								tag => {
									if (tag.tagName === "script") {
										tag.attributes.async = false;
									}
									return tag;
								}
							);
							callback(null, pluginArgs);
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[/<script defer="defer" src="app.js"><\/script>[\s]*<\/head>/],
			null,
			done,
			false,
			false
		);
	});

	it("allows events to remove an attribute by setting it to null", done => {
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).alterAssetTags.tapAsync(
						"HtmlWebpackPluginTest",
						(pluginArgs, callback) => {
							pluginArgs.assetTags.scripts = pluginArgs.assetTags.scripts.map(
								tag => {
									if (tag.tagName === "script") {
										tag.attributes.async = null;
									}
									return tag;
								}
							);
							callback(null, pluginArgs);
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[/<script defer="defer" src="app.js"><\/script>[\s]*<\/head>/],
			null,
			done,
			false,
			false
		);
	});

	it("allows events to remove an attribute by setting it to undefined", done => {
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).alterAssetTags.tapAsync(
						"HtmlWebpackPluginTest",
						(pluginArgs, callback) => {
							pluginArgs.assetTags.scripts = pluginArgs.assetTags.scripts.map(
								tag => {
									if (tag.tagName === "script") {
										tag.attributes.async = undefined;
									}
									return tag;
								}
							);
							callback(null, pluginArgs);
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[/<script defer="defer" src="app.js"><\/script>[\s]*<\/head>/],
			null,
			done,
			false,
			false
		);
	});

	it("provides the options to the afterEmit event", done => {
		let eventArgs;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).afterEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(pluginArgs, callback) => {
							eventArgs = pluginArgs;
							callback(null, pluginArgs);
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						foo: "bar"
					}),
					examplePlugin
				]
			},
			[/<script defer="defer" src="app.js"><\/script>[\s]*<\/head>/],
			null,
			() => {
				expect(eventArgs.plugin.options.foo).toBe("bar");
				done();
			},
			false,
			false
		);
	});

	it("provides the outputName to the afterEmit event", done => {
		let eventArgs;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).afterEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(pluginArgs, callback) => {
							eventArgs = pluginArgs;
							callback(null, pluginArgs);
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[/<script defer="defer" src="app.js"><\/script>[\s]*<\/head>/],
			null,
			() => {
				expect(eventArgs.outputName).toBe("index.html");
				done();
			},
			false,
			false
		);
	});

	it("fires the html-webpack-plugin-after-template-execution event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(
						compilation
					).afterTemplateExecution.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							callback();
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("fires the html-webpack-plugin-before-emit event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							callback();
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("fires the html-webpack-plugin-after-emit event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).afterEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							callback();
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			}
		);
	});

	it("allows to modify the html during html-webpack-plugin-before-emit event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							object.html += "Injected by plugin";
							callback();
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			["Injected by plugin"],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("allows to access all hooks from within a plugin", done => {
		let hookNames;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					hookNames = Object.keys(
						HtmlWebpackPlugin.getHooks(compilation)
					).sort();
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[],
			null,
			() => {
				expect(hookNames).toEqual([
					"afterEmit",
					"afterTemplateExecution",
					"alterAssetTagGroups",
					"alterAssetTags",
					"beforeAssetTagGeneration",
					"beforeEmit"
				]);
				done();
			},
			false,
			false
		);
	});

	it("allows to modify sequentially the html during html-webpack-plugin-before-emit event by edit the given arguments object", done => {
		let eventFiredForFirstPlugin = false;
		let eventFiredForSecondPlugin = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFiredForFirstPlugin = true;
							object.html += "Injected by first plugin";
							callback(null, object);
						}
					);
				});
			}
		};
		const secondExamplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFiredForSecondPlugin = true;
							object.html += " Injected by second plugin";
							callback(null);
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin, secondExamplePlugin]
			},
			["Injected by first plugin Injected by second plugin"],
			null,
			() => {
				expect(eventFiredForFirstPlugin).toBe(true);
				expect(eventFiredForSecondPlugin).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("allows to modify sequentially the html during html-webpack-plugin-before-emit event either by edit the given arguments object or by return a new object in the callback", done => {
		let eventFiredForFirstPlugin = false;
		let eventFiredForSecondPlugin = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFiredForFirstPlugin = true;
							const result = _.extend(object, {
								html: object.html + "Injected by first plugin"
							});
							callback(null, result);
						}
					);
				});
			}
		};
		const secondExamplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFiredForSecondPlugin = true;
							object.html += " Injected by second plugin";
							callback(null);
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name].js"
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin, secondExamplePlugin]
			},
			["Injected by first plugin Injected by second plugin"],
			null,
			() => {
				expect(eventFiredForFirstPlugin).toBe(true);
				expect(eventFiredForSecondPlugin).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("allows to modify sequentially the html during html-webpack-plugin-before-emit event by return a new object in the callback", done => {
		let eventFiredForFirstPlugin = false;
		let eventFiredForSecondPlugin = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFiredForFirstPlugin = true;
							const result = _.extend(object, {
								html: object.html + "Injected by first plugin"
							});
							callback(null, result);
						}
					);
				});
			}
		};
		const secondExamplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(compilation).beforeEmit.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFiredForSecondPlugin = true;
							const result = _.extend(object, {
								html: object.html + " Injected by second plugin"
							});
							callback(null, result);
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin, secondExamplePlugin]
			},
			["Injected by first plugin Injected by second plugin"],
			null,
			() => {
				expect(eventFiredForFirstPlugin).toBe(true);
				expect(eventFiredForSecondPlugin).toBe(true);
				done();
			}
		);
	});

	it("allows to modify the html during html-webpack-plugin-after-template-execution event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(
						compilation
					).afterTemplateExecution.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							object.bodyTags.push(
								HtmlWebpackPlugin.createHtmlTagObject("script", {
									src: "funky-script.js"
								})
							);
							object.html += "Injected by plugin";
							callback();
						}
					);
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			["Injected by plugin", '<script src="funky-script.js"'],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			},
			false,
			false
		);
	});

	it("allows to modify the html during html-webpack-plugin-before-asset-tag-generation event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(
						compilation
					).beforeAssetTagGeneration.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							object.assets.js.push("funky-script.js");
							callback();
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: false,
						template: path.join(__dirname, "fixtures/template.pug"),
						templateCompiler: pugTemplateCompiler
					}),
					examplePlugin
				]
			},
			['<script src="funky-script.js"'],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			}
		);
	});

	it("allows to inject files during html-webpack-plugin-before-asset-tag-generation event", done => {
		let eventFired = false;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					HtmlWebpackPlugin.getHooks(
						compilation
					).beforeAssetTagGeneration.tapAsync(
						"HtmlWebpackPluginTest",
						(object, callback) => {
							eventFired = true;
							object.assets.js.push("funky-script.js");
							callback();
						}
					);
				});
			}
		};
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			['<script defer="defer" src="funky-script.js"'],
			null,
			() => {
				expect(eventFired).toBe(true);
				done();
			}
		);
	});

	it("fires the events in the correct order", done => {
		const hookCallOrder = [
			"beforeAssetTagGeneration",
			"alterAssetTags",
			"alterAssetTagGroups",
			"afterTemplateExecution",
			"beforeEmit",
			"afterEmit"
		];
		let eventsFired = [];
		let hookLength = 0;
		const examplePlugin = {
			apply: function (compiler) {
				compiler.hooks.compilation.tap("HtmlWebpackPlugin", compilation => {
					const hooks = HtmlWebpackPlugin.getHooks(compilation);
					hookLength = hooks.length;
					// Hook into all hooks
					Object.keys(hooks).forEach(hookName => {
						hooks[hookName].tapAsync(
							"HtmlWebpackPluginTest",
							(object, callback) => {
								eventsFired.push(hookName);
								callback();
							}
						);
					});
				});
			}
		};

		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin(), examplePlugin]
			},
			[],
			null,
			() => {
				expect(hookLength).not.toBe(0);
				expect(eventsFired).toEqual(hookCallOrder);
				done();
			},
			false,
			false
		);
	});
	it.skip("works with commons chunk plugin", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					index: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				optimization: {
					splitChunks: {
						cacheGroups: {
							commons: {
								chunks: "initial",
								name: "common",
								enforce: true
							}
						}
					}
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[
				/<script defer="defer" src="common_bundle.js">[\s\S]*<script defer="defer" src="util_bundle.js">/,
				/<script defer="defer" src="common_bundle.js"[\s\S]*<script defer="defer" src="index_bundle.js">/
			],
			null,
			done
		);
	});

	it("adds a favicon", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="[^"]+\.ico">/],
			null,
			done
		);
	});

	it("adds a base tag with attributes", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						base: {
							href: "http://example.com/page.html",
							target: "_blank"
						}
					})
				]
			},
			[/<base href="http:\/\/example\.com\/page\.html" target="_blank">/],
			null,
			done
		);
	});

	it("adds a base tag short syntax", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						base: "http://example.com/page.html"
					})
				]
			},
			[/<base href="http:\/\/example\.com\/page\.html">/],
			null,
			done
		);
	});

	it("adds a meta tag", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						meta: {
							viewport: {
								name: "viewport",
								content: "width=device-width, initial-scale=1, shrink-to-fit=no"
							}
						}
					})
				]
			},
			[
				/<meta name="viewport" content="width=device-width,initial-scale=1,shrink-to-fit=no">/
			],
			null,
			done
		);
	});

	it("adds a meta tag with short notation", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						meta: {
							viewport: "width=device-width, initial-scale=1, shrink-to-fit=no"
						}
					})
				]
			},
			[
				/<meta name="viewport" content="width=device-width,initial-scale=1,shrink-to-fit=no">/
			],
			null,
			done
		);
	});

	it("adds a favicon with publicPath set to /some/", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/some/"
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="\/some\/+[^"]+\.ico">/],
			null,
			done
		);
	});

	it("adds a favicon with publicPath set to /some", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/some"
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="\/some\/+[^"]+\.ico">/],
			null,
			done
		);
	});

	it("adds a favicon with publicPath set to some/", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "some/"
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="some\/+[^"]+\.ico">/],
			null,
			done
		);
	});

	it("adds a favicon with publicPath undefined", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="[^"]+\.ico">/],
			null,
			done
		);
	});

	it("adds a favicon with publicPath undefined", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico"),
						filename: path.resolve(OUTPUT_DIR, "subfolder", "test.html")
					})
				]
			},
			[/<link rel="icon" href="\.\.\/[^"]+\.ico">/],
			path.join("subfolder", "test.html"),
			done
		);
	});

	it.skip("adds a favicon with a publicPath set to /[hash]/ and replaces the hash", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "/[hash]/",
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="\/[a-z0-9]{20}\/favicon\.ico">/],
			null,
			done
		);
	});

	it.skip("adds a favicon with a publicPath set to [hash]/ and replaces the hash", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					publicPath: "[hash]/",
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="[a-z0-9]{20}\/favicon\.ico">/],
			null,
			done
		);
	});

	it("adds a favicon with inject enabled", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="[^"]+\.ico">/],
			null,
			done
		);
	});

	it("adds a favicon with xhtml enabled", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						xhtml: true,
						minify: {
							keepClosingSlash: true
						},
						favicon: path.join(__dirname, "fixtures/favicon.ico")
					})
				]
			},
			[/<link rel="icon" href="[^"]+\.ico"\/>/],
			null,
			done
		);
	});

	it.skip("shows an error if the favicon could not be load", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				optimization: {
					emitOnErrors: true
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: true,
						favicon: path.join(__dirname, "fixtures/does_not_exist.ico")
					})
				]
			},
			["Error: HtmlWebpackPlugin: could not load file"],
			null,
			done,
			true
		);
	});

	it.skip("works with webpack bannerplugin", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new webpack.BannerPlugin("Copyright and such."),
					new HtmlWebpackPlugin()
				]
			},
			["<html"],
			null,
			done
		);
	});

	it.skip("shows an error when a template fails to load", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(
							__dirname,
							"fixtures/non-existing-template.html"
						)
					})
				]
			},
			["Child compilation failed:\n  Module not found:"],
			null,
			done,
			true
		);
	});

	it.skip("should sort the chunks in auto mode", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					util: path.join(__dirname, "fixtures/util.js"),
					index: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				optimization: {
					splitChunks: {
						cacheGroups: {
							commons: {
								chunks: "initial",
								name: "common",
								enforce: true
							}
						}
					}
				},
				plugins: [
					new HtmlWebpackPlugin({
						chunksSortMode: "auto"
					})
				]
			},
			[
				/(<script defer="defer" src="common_bundle.js">.+<script defer="defer" src="util_bundle.js">.+<script defer="defer" src="index_bundle.js">)|(<script defer="defer" src="common_bundle.js">.+<script defer="defer" src="index_bundle.js">.+<script defer="defer" src="util_bundle.js">)/
			],
			null,
			done
		);
	});

	it("should sort the chunks in custom (reverse alphabetical) order", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					b: path.join(__dirname, "fixtures/index.js"),
					c: path.join(__dirname, "fixtures/util.js"),
					a: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						chunksSortMode: function (a, b) {
							if (a < b) {
								return 1;
							}
							if (a > b) {
								return -1;
							}
							return 0;
						}
					})
				]
			},
			[
				/<script defer="defer" src="c_bundle.js">.+<script defer="defer" src="b_bundle.js">.+<script defer="defer" src="a_bundle.js">/
			],
			null,
			done
		);
	});

	it.skip("should sort manually by the chunks", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					b: path.join(__dirname, "fixtures/util.js"),
					a: path.join(__dirname, "fixtures/theme.js"),
					d: path.join(__dirname, "fixtures/util.js"),
					c: path.join(__dirname, "fixtures/theme.js")
				},
				output: {
					path: OUTPUT_DIR,
					filename: "[name]_bundle.js"
				},
				module: {
					rules: [{ test: /\.css$/, loader: "css-loader" }]
				},
				optimization: {
					splitChunks: {
						cacheGroups: {
							commons: {
								chunks: "initial",
								name: "common",
								enforce: true
							}
						}
					}
				},
				plugins: [
					new HtmlWebpackPlugin({
						chunksSortMode: "manual",
						chunks: ["common", "a", "b", "c"]
					})
				]
			},
			[
				/<script defer="defer" src="common_bundle.js">.+<script defer="defer" src="a_bundle.js">.+<script defer="defer" src="b_bundle.js">.+<script defer="defer" src="c_bundle.js">/
			],
			null,
			done
		);
	});

	it.skip("should add the webpack compilation object as a property of the templateParam object", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/templateParam.js"),
						inject: false
					})
				]
			},
			['templateParams keys: "compilation,webpackConfig,htmlWebpackPlugin"'],
			null,
			done
		);
	});

	it.skip("should allow to disable template parameters", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/templateParam.js"),
						inject: false,
						templateParameters: false
					})
				]
			},
			['templateParams keys: ""'],
			null,
			done
		);
	});

	it.skip("should allow to set specific template parameters", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/templateParam.js"),
						inject: false,
						templateParameters: { foo: "bar" }
					})
				]
			},
			[
				'templateParams keys: "compilation,webpackConfig,htmlWebpackPlugin,foo"'
			],
			null,
			done
		);
	});

	it.skip("should allow to set specific template parameters using a function", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/templateParam.js"),
						inject: false,
						templateParameters: function () {
							return { foo: "bar" };
						}
					})
				]
			},
			['templateParams keys: "foo"'],
			null,
			done
		);
	});

	it.skip("should allow to set specific template parameters using a async function", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js"
				},
				plugins: [
					new HtmlWebpackPlugin({
						template: path.join(__dirname, "fixtures/templateParam.js"),
						inject: false,
						templateParameters: function () {
							return Promise.resolve({ foo: "bar" });
						}
					})
				]
			},
			['templateParams keys: "foo"'],
			null,
			done
		);
	});

	it("should not treat templateContent set to an empty string as missing", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: { app: path.join(__dirname, "fixtures/index.js") },
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						templateContent: ""
					})
				]
			},
			[/^<head><script defer="defer" src="app\.js"><\/script><\/head>$/],
			null,
			done
		);
	});

	it("allows you to inject the assets into the body of the given spaced closing tag template", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: "body",
						template: path.join(__dirname, "fixtures/spaced_plain.html")
					})
				]
			},
			[
				/<body>[\s]*<script defer="defer" src="main.js"><\/script>[\s]*<\/body>/
			],
			null,
			done
		);
	});

	it("allows you to inject the assets into the head of the given spaced closing tag template", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: "head",
						template: path.join(__dirname, "fixtures/spaced_plain.html")
					})
				]
			},
			[/<script defer="defer" src="main.js"><\/script>[\s]*<\/head>/],
			null,
			done
		);
	});

	it("should minify by default when mode is production", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[/<!doctype html><html><head><meta charset="utf-8">/],
			null,
			done
		);
	});

	it("should not minify by default when mode is development", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin()]
			},
			[/<!DOCTYPE html>\s+<html>\s+<head>\s+<meta charset="utf-8">/],
			null,
			done
		);
	});

	it("should minify in production if options.minify is true", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ minify: true })]
			},
			[/<!doctype html><html><head><meta charset="utf-8">/],
			null,
			done
		);
	});

	it("should minify in development if options.minify is true", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ minify: true })]
			},
			[/<!doctype html><html><head><meta charset="utf-8">/],
			null,
			done
		);
	});

	it("should not minify in production if options.minify is false", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ minify: false })]
			},
			[/<!DOCTYPE html>\s+<html>\s+<head>\s+<meta charset="utf-8">/],
			null,
			done
		);
	});

	it("should not minify in development if options.minify is false", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({ minify: false })]
			},
			[/<!DOCTYPE html>\s+<html>\s+<head>\s+<meta charset="utf-8">/],
			null,
			done
		);
	});

	it("should allow custom minify options and not merge them with the defaults", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						minify: {
							useShortDoctype: true
						}
					})
				]
			},
			[/<!doctype html>\s+<html>\s+<head>\s+<meta charset="utf-8">/],
			null,
			done
		);
	});

	it('should allow to inject scripts with a defer="defer" attribute', done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "defer"
					})
				]
			},
			[/<script defer="defer" .+<body>/],
			null,
			done
		);
	});

	it('should allow to inject scripts with a type="module" attribute', done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "module"
					})
				]
			},
			[/<script type="module" src="main.js"><\/script>.+<body>/],
			null,
			done
		);
	});

	it('should allow to inject scripts with a defer="defer" attribute to the body', done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "defer",
						inject: "body"
					})
				]
			},
			[/<body>.*<script defer="defer"/],
			null,
			done
		);
	});

	it('should allow to inject scripts with a defer="defer" in front of styles', done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "defer"
					})
				]
			},
			[/<script defer="defer".+<link href="main.css"/],
			null,
			done
		);
	});

	it("should keep closing slashes from the template", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "defer",
						templateContent: "<html><body> <selfclosed /> </body></html>"
					})
				]
			},
			[/<selfclosed\/>/],
			null,
			done
		);
	});

	it("should add the javascript assets to the head for inject:true with scriptLoading:defer", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "defer",
						inject: true
					})
				]
			},
			[
				'<script defer="defer" src="main.js"></script><link href="main.css" rel="stylesheet"></head>'
			],
			null,
			done
		);
	});

	it("should allow to use headTags and bodyTags directly in string literals", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "blocking",
						inject: false,
						templateContent: ({ htmlWebpackPlugin }) => `
            <html>
              <head>${htmlWebpackPlugin.tags.headTags}</head>
              <body>${htmlWebpackPlugin.tags.bodyTags}</body>
            </html>
            `
					})
				]
			},
			[
				'<head><link href="main.css" rel="stylesheet"></head>',
				'<script src="main.js"></script></body>'
			],
			null,
			done
		);
	});

	it("should add the javascript assets to the head for inject:true with scriptLoading:defer", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						scriptLoading: "defer",
						inject: true
					})
				]
			},
			[
				'<script defer="defer" src="main.js"></script><link href="main.css" rel="stylesheet"></head>'
			],
			null,
			done
		);
	});

	it("should allow to use headTags and bodyTags directly in string literals", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/theme.js"),
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						inject: false,
						templateContent: ({ htmlWebpackPlugin }) => `
            <html>
              <head>${htmlWebpackPlugin.tags.headTags}</head>
              <body>${htmlWebpackPlugin.tags.bodyTags}</body>
            </html>
            `
					})
				]
			},
			[
				'<head><script defer="defer" src="main.js"></script><link href="main.css" rel="stylesheet"></head>'
			],
			null,
			done
		);
	});

	it.skip("should allow to use experiments:{outputModule:true}", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js",
					module: true
				},
				experiments: { outputModule: true },
				plugins: [new HtmlWebpackPlugin({})]
			},
			['<script defer="defer" src="index_bundle.js"></script>'],
			null,
			done
		);
	});

	it.skip("generates relative path for asset/resource", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js",
					assetModuleFilename: "assets/demo[ext]"
				},
				module: {
					rules: [{ test: /\.png$/, type: "asset/resource" }]
				},
				plugins: [
					new HtmlWebpackPlugin({
						template:
							"html-loader!" + path.join(__dirname, "fixtures/logo.html"),
						filename: "demo/index.js"
					})
				]
			},
			['<img src="../assets/demo.png'],
			"demo/index.js",
			done
		);
	});

	it.skip("uses the absolute path for asset/resource", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: path.join(__dirname, "fixtures/index.js"),
				output: {
					path: OUTPUT_DIR,
					filename: "index_bundle.js",
					assetModuleFilename: "assets/demo[ext]"
				},
				module: {
					rules: [{ test: /\.png$/, type: "asset/resource" }]
				},
				plugins: [
					new HtmlWebpackPlugin({
						template:
							"html-loader!" + path.join(__dirname, "fixtures/logo.html"),
						filename: "demo/index.js",
						publicPath: "/foo/"
					})
				]
			},
			['<img src="/foo/assets/demo.png'],
			"demo/index.js",
			done
		);
	});

	it("generates an html file if entry is empty", done => {
		testHtmlPlugin(
			{
				mode: "development",
				entry: {},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [new HtmlWebpackPlugin({})]
			},
			["<body>"],
			null,
			done
		);
	});

	it("allows to set custom loader interpolation settings", done => {
		testHtmlPlugin(
			{
				mode: "production",
				entry: {
					app: path.join(__dirname, "fixtures/index.js")
				},
				output: {
					path: OUTPUT_DIR
				},
				plugins: [
					new HtmlWebpackPlugin({
						title: "Interpolation Demo",
						template: path.join(__dirname, "fixtures/interpolation.html"),
						templateCompiler: {
							...defaultTemplateCompiler,
							options: { interpolate: /\{%=([\s\S]+?)%\}/g }
						}
					})
				]
			},
			["Interpolation Demo"],
			null,
			() => {
				done();
			}
		);
	});
});
