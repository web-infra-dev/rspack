module.exports = {
	resolve: {
		extensions: [".js"],
		modules: ["vendor"]
	},
	experiments: {
		useInputFileSystem: [/regexp-dfa/]
	},
	plugins: [
		{
			apply(compiler) {
				// A persistent-cache snapshot also stats entries under the context.
				if (compiler.options.cache?.type === "persistent") return;
				const original = compiler.inputFileSystem;
				const input = Object.create(original);
				let statUnrelatedDirectory = false;

				input.stat = (path, ...args) => {
					const normalized = path.toString().replace(/\\/g, "/");
					if (normalized.endsWith("/pruning/unrelated")) {
						statUnrelatedDirectory = true;
					}
					return original.stat(path, ...args);
				};
				compiler.inputFileSystem = input;
				compiler.hooks.beforeCompile.tap("RegExpDfa", () => {
					compiler.inputFileSystem = input;
				});
				compiler.hooks.afterCompile.tap("RegExpDfa", () => {
					expect(statUnrelatedDirectory).toBe(false);
				});
			}
		}
	]
};
