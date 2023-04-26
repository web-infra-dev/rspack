// @ts-nocheck
import { Compiler, getNormalizedRspackOptions, rspack } from "../src";
const path = require("path");
const { IgnorePlugin } = require("webpack");

class Plugin implements RspackPluginInstance {
	name = "a";

	apply(compiler: Compiler) {
		// Wait for configuration preset plugions to apply all configure webpack defaults
		compiler.hooks.compilation.tap("a", com => {
			com.normalModuleFactory.hooks.beforeResolve.tap("a", (...args) => {
				console.log(...args);
			});
			com.contextModuleFactory.hooks.beforeResolve.tap("a", (...args) => {
				console.log(...args);
			});
		});
	}
}

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
			stats.logs = logs;
			c.close(err => {
				if (err) return callback(err);
				callback(stats, files, compilation);
			});
		});
	}

	let compiler: Compiler;
	afterEach(callback => {
		if (compiler) {
			compiler.close(callback);
			compiler = undefined;
		} else {
			callback();
		}
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
					stats.modules.filter(module =>
						module.identifier.startsWith("missing")
					).length
				).toBe(3);
				done();
			}
		);
	});
});
