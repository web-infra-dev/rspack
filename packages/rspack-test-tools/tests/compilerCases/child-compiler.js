const path = require("path");
const { createFsFromVolume, Volume } = require("memfs");
const { EntryPlugin } = require("@rspack/core");

let stubCompilerName = jest.fn();

let cachedChildCompiler;
let cachedChildCompilation;
let stubCompiler = jest.fn();
let stubCompilation = jest.fn();

const CHILD_COMPILER_NAME = "NEW_CHILD_COMPILER";

let inChild = false;
class MyPlugin {
	/**
	 *
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.make.tapAsync("Plugin", (compilation, callback) => {
			let childCompiler = compilation.createChildCompiler(
				CHILD_COMPILER_NAME,
				{},
				[]
			);
			new EntryPlugin(compiler.context, "./d").apply(childCompiler);

			inChild = true;

			// SAFETY: It's safe to cache `childCompiler` here as we never access its inner content.
			// NEVER do this in any userland code.
			cachedChildCompiler = childCompiler;

			childCompiler.hooks.compilation.tap("Plugin", compilation => {
				// SAFETY: It's safe to cache `childCompiler` here as we never access its inner content.
				// NEVER do this in any userland code.
				cachedChildCompilation = compilation;
			});

			childCompiler.runAsChild((...args) => {
				inChild = false;
				callback(...args);
			});
		});
	}
}

const outputFileSystem = createFsFromVolume(new Volume());
module.exports = {
	description:
		"should pass the new compiler and compilation instance in loader",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()],
			module: {
				rules: [
					{
						test: /\.js$/,
						loader: path.resolve(context.getSource(), "./callback-loader.js"),
						options: {
							callback(loaderContext) {
								if (inChild) {
									stubCompilerName(loaderContext._compiler.name);
									stubCompiler(loaderContext._compiler);
									stubCompilation(loaderContext._compilation);
								}
							}
						}
					}
				]
			}
		};
	},
	async compiler(context, compiler) {
		compiler.outputFileSystem = outputFileSystem;
	},
	async check() {
		expect(stubCompilerName).toHaveBeenCalledTimes(1);
		expect(stubCompilerName).toHaveBeenCalledWith(CHILD_COMPILER_NAME);
		expect(stubCompiler).toHaveBeenCalledWith(cachedChildCompiler);
		expect(stubCompilation).toHaveBeenCalledWith(cachedChildCompilation);
	}
};
