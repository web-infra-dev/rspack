const path = require("path");
const { createFsFromVolume, Volume } = require("memfs");
const webpack = require("@rspack/core");

const assetsNames = assets => assets.map(asset => asset.name);

describe("TestMemoryFS", () => {
	it("should preserve asset even if not emitted", done => {
		const casesDirectory = path.resolve(__dirname, "cases");
		const directoryForCase = path.resolve(
			casesDirectory,
			"publicpath-default-auto"
		);
		// eslint-disable-next-line import/no-dynamic-require, global-require
		const webpackConfig = require(path.resolve(
			directoryForCase,
			"webpack.config.js"
		));
		const compiler = webpack({
			...webpackConfig,
			mode: "development",
			context: directoryForCase,
			cache: false
		});

		compiler.outputFileSystem = createFsFromVolume(new Volume());

		compiler.run((err1, stats1) => {
			if (err1) {
				done(err1);

				return;
			}

			// CHANGE: The compilation instance of Rspack will be dropped on the Rust side after compilation.
			// So we should obtain all the assets information after the next time the compile. 
			const names1 = assetsNames(stats1.compilation.getAssets());

			compiler.run((err2, stats2) => {
				if (err2) {
					done(err2);

					return;
				}

				const names2 = assetsNames(stats2.compilation.getAssets());
				expect(names1).toEqual(names2);

				done();
			});
		});
	});
});
