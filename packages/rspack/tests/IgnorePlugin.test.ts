import { Compiler, getNormalizedRspackOptions, rspack } from "../src";
import path from "path";
import { IgnorePlugin } from "webpack";

describe("Ignore Plugin", () => {
	jest.setTimeout(20000);
	function compile(entry: string, options, callback) {
		const noOutputPath = !options.output || !options.output.path;

		options = getNormalizedRspackOptions(options);

		if (!options.mode) options.mode = "development";
		options.entry = entry;
		options.context = path.join(__dirname, "ignorePlugin");
		if (noOutputPath) options.output.path = "/";
		options.optimization = {
			minimize: false
		};
		options.plugins = [
			new IgnorePlugin({
				checkResource: (resource, request) => {
					if (resource.includes("zh") || resource.includes("globalIndex")) {
						return true;
					}
					return false;
				}
			})
		];
		options.cache = true;
		const logs = {
			mkdir: [],
			writeFile: []
		};

		const c = rspack(options);
		const files = {};
		c.run((err, stats) => {
			if (err) throw err;
			expect(typeof stats).toBe("object");
			expect(typeof stats).toBe("object");
			// const compilation = stats?.compilation;
			// stats = stats.toJson);
			c.close(err => {
				console.log("close");

				if (err) return callback(err);
				callback(
					stats?.toJson({
						modules: true,
						reasons: true
					}),
					files
				);
			});
		});
	}

	afterEach(callback => {
		callback();
	});

	it("should be ignore module", done => {
		const outputDist = "dist/ignorePlugin";
		compile(
			"./index.js",
			{
				output: {
					path: outputDist,
					filename: "index.js"
				}
			},
			(stats, files) => {
				expect(
					stats.modules
						.filter(module => module.identifier.startsWith("missing"))
						.map(module => module.identifier.split("ignorePlugin")?.[1])
				).toMatchInlineSnapshot(`
			[
			  "./zh",
			  "./zh.js",
			  "./globalIndex.js",
			]
		`);
				done();
			}
		);
	});
});
