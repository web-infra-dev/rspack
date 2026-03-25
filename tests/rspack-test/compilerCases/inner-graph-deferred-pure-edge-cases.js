const fs = require("node:fs");
const path = require("node:path");

const CASE_DIR = "inner-graph-deferred-pure-edge-cases";
const OUTPUT_DIR = "output";

function readOutput(context) {
	return fs.readFileSync(context.getDist(path.join(OUTPUT_DIR, "main.js")), "utf-8");
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description:
		"should keep unsupported deferred pure edge cases conservative",
	options(context) {
		return {
			mode: "production",
			target: "node",
			context: path.resolve(__dirname, "../fixtures", CASE_DIR),
			entry: "./index.js",
			experiments: {
				advancedTreeShaking: true
			},
			module: {
				rules: [
					{
						test: /dep-direct\.js$/,
						parser: {
							sideEffectsFree: ["pureDirect"]
						}
					},
					{
						test: /dep-default-export\.js$/,
						parser: {
							sideEffectsFree: ["default"]
						}
					},
					{
						test: /dep-reexport\.js$/,
						parser: {
							sideEffectsFree: ["pureReexport"]
						}
					},
					{
						test: /dep-star\.js$/,
						parser: {
							sideEffectsFree: ["pureStar"]
						}
					},
					{
						test: /dep-all-pure-a\.js$/,
						parser: {
							sideEffectsFree: ["pureAllA"]
						}
					},
					{
						test: /dep-all-pure-b\.js$/,
						parser: {
							sideEffectsFree: ["pureAllB"]
						}
					},
					{
						test: /dep-mixed-a\.js$/,
						parser: {
							sideEffectsFree: ["pureMixedA"]
						}
					}
				]
			},
			optimization: {
				sideEffects: true,
				innerGraph: true,
				usedExports: true,
				concatenateModules: false
			},
			output: {
				path: context.getDist(OUTPUT_DIR),
				filename: "main.js",
				clean: true
			}
		};
	},
	async compiler(_, compiler) {
		compiler.outputFileSystem = fs;
	},
	async build(context) {
		await context.getCompiler().build();
		context.setValue("output", readOutput(context));
	},
	async check({ context }) {
		const output = context.getValue("output");

		expect(output).not.toContain("direct-simple-marker");
		expect(output).not.toContain("direct-alias-marker");
		expect(output).not.toContain("default-alias-marker");
		expect(output).not.toContain("reexport-alias-marker");
		expect(output).not.toContain("star-reexport-marker");
		expect(output).toContain("all-pure-a-marker");
		expect(output).toContain("all-pure-b-marker");
		expect(output).toContain("mixed-pure-a-marker");
		expect(output).toContain("mixed-impure-b-marker");
	}
};
