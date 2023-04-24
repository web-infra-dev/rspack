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

	it("should be warning tsconfig", done => {
		console.log = jest.fn();
		createCompiler("demo1");
		expect(console.log.mock.calls[0][0]).toMatchInlineSnapshot(`
		"[33mInvalid file object. JSON schema for the TypeScript compiler's configuration has been initialized using a file object that does not match the API schema.
		 - file.compilerOptions misses the property 'verbatimModuleSyntax'. Should be:
		   true
		   -> SWC warning more info see: https://swc.rs/docs/migrating-from-tsc#esmoduleinterop-true 
		   Do not transform or elide any imports or exports not marked as type-only, ensuring they are written in the output file's format based on the 'module' setting. [0m"
	`);
		done();
	});
	it("should be success tsconfig", done => {
		console.log = jest.fn();
		createCompiler("demo2");
		expect(console.log.mock.calls).toHaveLength(0);
		done();
	});
});
