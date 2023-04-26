// @ts-nocheck
import { readdirSync } from "fs";
import { Compiler, getNormalizedRspackOptions, rspack } from "../src";
const path = require("path");

describe("Output", () => {
	jest.setTimeout(20000);
	function compile(entry: string, options, callback) {
		const noOutputPath = !options.output || !options.output.path;

		options = getNormalizedRspackOptions(options);

		if (!options.mode) options.mode = "production";
		options.entry = entry;
		options.context = path.join(__dirname, "fixtures");
		if (noOutputPath) options.output.path = "/";
		options.optimization = {
			minimize: false
		};
		options.plugins = [new Plugin()];
		options.cache = true;
		const logs = {
			mkdir: [],
			writeFile: []
		};

		const c = rspack(options);
		const files = {};
		// c.outputFileSystem = {
		// 	mkdir(path, callback) {
		// 		logs.mkdir.push(path);
		// 		const err = new Error();
		// 		err.code = "EEXIST";
		// 		callback(err);
		// 	},
		// 	writeFile(name, content, callback) {
		// 		logs.writeFile.push(name, content);
		// 		files[name] = content.toString("utf-8");
		// 		callback();
		// 	},
		// 	stat(path, callback) {
		// 		callback(new Error("ENOENT"));
		// 	}
		// };
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
						expect(readdirSync(outputDist)).toEqual(["hell2.js"]);
						done();
					}
				);
			}
		);
	});
});
