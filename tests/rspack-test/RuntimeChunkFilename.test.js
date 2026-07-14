const fs = require("node:fs");
const path = require("node:path");
const { rspack } = require("@rspack/core");

const fixtureRoot = path.resolve(__dirname, "js/runtime-chunk-filename");

function write(root, filename, content) {
	const file = path.join(root, filename);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
}

describe("chunk filename runtime requirements", () => {
	afterAll(() => {
		fs.rmSync(fixtureRoot, { force: true, recursive: true });
	});

	it("defines the full-hash runtime for an extracted async CSS filename", async () => {
		const root = path.join(fixtureRoot, "extract-css-fullhash");
		fs.rmSync(root, { force: true, recursive: true });
		write(
			root,
			"src/index.js",
			"import(/* webpackChunkName: 'async' */ './async');\n"
		);
		write(root, "src/async.js", "import './async.css';\n");
		write(root, "src/async.css", ".async { color: red; }\n");

		const compiler = rspack({
			context: root,
			mode: "development",
			target: "web",
			devtool: false,
			entry: "./src/index.js",
			output: {
				path: path.join(root, "dist"),
				filename: "main.js",
				chunkFilename: "[name].js"
			},
			module: {
				rules: [
					{
						test: /\.css$/,
						type: "javascript/auto",
						use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
					}
				]
			},
			optimization: { minimize: false, splitChunks: false },
			plugins: [
				new rspack.CssExtractRspackPlugin({
					filename: "[name].css",
					chunkFilename: "[name].[fullhash].css"
				})
			]
		});

		try {
			const compilation = await new Promise((resolve, reject) => {
				compiler.run((error, stats) => {
					if (error) return reject(error);
					if (stats.hasErrors()) {
						return reject(
							new Error(stats.toString({ all: false, errors: true }))
						);
					}
					resolve(stats.compilation);
				});
			});
			const assets = compilation.getAssets();
			const runtime = compilation
				.getAsset("main.js")
				.source.source()
				.toString();

			expect(
				assets.some(({ name }) => /^async\.[a-f0-9]+\.css$/.test(name))
			).toBe(true);
			expect(runtime).toContain("miniCssF");
			expect(runtime).toContain("__webpack_require__.h()");
			expect(runtime).toMatch(/__webpack_require__\.h\s*=/);
		} finally {
			await new Promise((resolve, reject) => {
				compiler.close(error => (error ? reject(error) : resolve()));
			});
		}
	});
});
