import path from "path";

import { createFsFromVolume, Volume } from "memfs";
import webpack from "../../";

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

			compiler.run((err2, stats2) => {
				if (err2) {
					done(err2);

					return;
				}

				expect(assetsNames(stats1.compilation.getAssets())).toEqual(
					assetsNames(stats2.compilation.getAssets())
				);

				done();
			});
		});
	});
});
