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

describe("insert option", () => {
	it(`should work without insert option`, async () => {
		const compiler = getCompiler(
			"insert.js",
			{},
			{
				mode: "none",
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

		runInJsDom("main.bundle.js", compiler, stats, dom => {
			expect(dom.serialize()).toMatchSnapshot("DOM");
		});

		expect(getWarnings(stats)).toMatchSnapshot("warnings");
		expect(getErrors(stats)).toMatchSnapshot("errors");
	});

	it(`should work when insert option is string`, async () => {
		const compiler = getCompiler(
			"insert.js",
			{},
			{
				mode: "none",
				output: {
					publicPath: "",
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},
				plugins: [
					new CssExtractRspackPlugin({
						filename: "[name].css",
						insert: "#existing-style"
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

	it(`should work when insert option is function`, async () => {
		const compiler = getCompiler(
			"insert.js",
			{},
			{
				mode: "none",
				output: {
					publicPath: "",
					path: path.resolve(__dirname, "../outputs"),
					filename: "[name].bundle.js"
				},
				plugins: [
					new CssExtractRspackPlugin({
						filename: "[name].css",
						// eslint-disable-next-line
						insert: function (linkTag) {
							const reference = document.querySelector("#existing-style");
							if (reference) {
								reference.parentNode.insertBefore(linkTag, reference);
							}
						}
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
});
