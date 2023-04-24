// @ts-nocheck
import { Compiler, getNormalizedRspackOptions, rspack } from "../src";
const path = require("path");

describe("TsconfigPath", () => {
	jest.setTimeout(20000);
	function createCompiler(dir: string) {
		let options = {};
		options = getNormalizedRspackOptions(options);
		if (!options.mode) options.mode = "production";
		options.entry = "./index.js";
		options.context = path.join(__dirname, "tsconfig", dir);
		options.optimization = {
			minimize: false
		};
		options.cache = true;
		rspack(options);
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
		createCompiler("demo1");
		done();
	});
});
