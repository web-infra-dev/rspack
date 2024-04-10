const path = require("path");

const { rspack: webpack } = require("../../dist");

describe("IgnoreOrder", () => {
	it("should emit warnings", done => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "ignoreOrderFalse");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const compiler = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: false,
			experiments: {
				css: false,
				rspackFuture: {
					newTreeshaking: true
				}
			}
		});
		compiler.run((err1, stats) => {
			expect(stats.hasWarnings()).toBe(true);
			done();
		});
	});

	it("should not emit warnings", done => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(casesDirectory, "ignoreOrder");
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const compiler = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: false,
			experiments: {
				css: false,
				rspackFuture: {
					newTreeshaking: true
				}
			}
		});
		compiler.run((err1, stats) => {
			expect(stats.hasWarnings()).toBe(false);
			done();
		});
	});
});
