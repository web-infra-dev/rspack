const path = require("path");
const fs = require("fs");
const rspack = require("@rspack/core");

describe("Output", () => {
	function compile(entry, options, callback) {
		const noOutputPath = !options.output || !options.output.path;
		options = rspack.config.getNormalizedRspackOptions(options);
		if (!options.mode) options.mode = "production";
		options.entry = entry;
		options.context = path.join(__dirname, "../fixtures");
		if (noOutputPath) options.output.path = "/";
		options.optimization = {
			minimize: false
		};
		options.cache = true;
		const c = rspack(options);
		const files = {};
		c.hooks.compilation.tap("CompilerTest", compilation => {
			compilation.bail = true;
		});
		c.run((err, stats) => {
			if (err) throw err;
			expect(typeof stats).toBe("object");
			const compilation = stats.compilation;
			stats = stats.toJson({
				modules: true,
				reasons: true
			});
			expect(typeof stats).toBe("object");
			expect(stats).toHaveProperty("errors");
			expect(Array.isArray(stats.errors)).toBe(true);
			if (stats.errors.length > 0) {
				expect(stats.errors[0]).toBeInstanceOf(Error);
				throw stats.errors[0];
			}
			c.close(err => {
				if (err) return callback(err);
				callback(stats, files, compilation);
			});
		});
	}

	let compiler;
	afterEach(callback => {
		if (compiler) {
			compiler.close(callback);
			compiler = undefined;
		} else {
			callback();
		}
	});

	it("should be cleared the build directory", done => {
		const outputDist = "dist/output";
		compile(
			"./a",
			{
				output: {
					path: outputDist,
					filename: "hell1.js"
				}
			},
			() => {
				compile(
					"./a",
					{
						output: {
							clean: true,
							path: outputDist,
							filename: "hell2.js"
						}
					},
					() => {
						expect(fs.readdirSync(outputDist)).toEqual(["hell2.js"]);
						done();
					}
				);
			}
		);
	});
});
