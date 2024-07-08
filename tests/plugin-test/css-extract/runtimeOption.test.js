/* eslint-env browser */
const path = require("path");
const { CssExtractRspackPlugin } = require("@rspack/core");
const {
	compile,
	getCompiler,
	getErrors,
	getWarnings,
	runInJsDom
} = require("./helpers/index");

describe("noRuntime option", () => {
	it.only("should work without the 'runtime' option", async () => {
		const compiler = getCompiler(
			"attributes.js",
			{},
			{
				output: {
					publicPath: "",
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},
				plugins: [
					new CssExtractRspackPlugin({
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		runInJsDom("main.bundle.js", compiler, stats, (dom, bundle) => {
			expect(dom.serialize()).toMatchSnapshot("DOM");
			expect(bundle).toContain("webpack/runtime/css loading");
			expect(bundle).toContain("webpack/runtime/get mini-css chunk filename");
		});

		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it("should work when the 'runtime' option is 'false'", async () => {
		const compiler = getCompiler(
			"attributes.js",
			{},
			{
				output: {
					publicPath: "",
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},
				plugins: [
					new CssExtractRspackPlugin({
						runtime: false,
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		runInJsDom("main.bundle.js", compiler, stats, (dom, bundle) => {
			expect(dom.serialize()).toMatchSnapshot("DOM");
			expect(bundle).not.toContain("webpack/runtime/css loading");
			expect(bundle).not.toContain(
				"webpack/runtime/get mini-css chunk filename"
			);
		});

		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it("should work when the 'runtime' option is 'true'", async () => {
		const compiler = getCompiler(
			"attributes.js",
			{},
			{
				output: {
					publicPath: "",
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},
				plugins: [
					new CssExtractRspackPlugin({
						runtime: true,
						filename: "[name].css"
					})
				]
			}
		);
		const stats = await compile(compiler);

		runInJsDom("main.bundle.js", compiler, stats, (dom, bundle) => {
			expect(dom.serialize()).toMatchSnapshot("DOM");
			expect(bundle).toContain("webpack/runtime/css loading");
			expect(bundle).toContain("webpack/runtime/get mini-css chunk filename");
		});

		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});
});
