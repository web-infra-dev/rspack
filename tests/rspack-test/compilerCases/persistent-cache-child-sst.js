const { promisify } = require("node:util");

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
	description: "should persist parent and child compiler caches without storage conflicts",
	options(context) {
		return {
			experiments: { newCache: true },
			cache: {
				type: "persistent",
				maxMemoryGenerations: 0,
				storage: { type: "filesystem", location: context.getDist("cache") }
			}
		};
	},
	async build(context, compiler) {
		const run = async parent => {
			const built = { parent: 0, child: 0 };
			const track = (target, name) => {
				target.hooks.thisCompilation.tap("ChildSstTest", compilation => {
					compilation.hooks.buildModule.tap("ChildSstTest", () => built[name]++);
				});
			};
			track(parent, "parent");
			parent.hooks.make.tapAsync("ChildSstTest", (compilation, callback) => {
				const child = compilation.createChildCompiler("child", { filename: "child.js" }, [
					new parent.rspack.EntryPlugin(parent.context, "./b", { name: "main" })
				]);
				track(child, "child");
				child.runAsChild(callback);
			});
			try {
				const stats = await promisify(parent.run.bind(parent))();
				expect(stats.hasErrors()).toBe(false);
				return built;
			} finally {
				await promisify(parent.close.bind(parent))();
			}
		};

		expect(await run(compiler)).toEqual({ parent: 1, child: 1 });
		// Reopening the same directory must recover both compilers' data from disk.
		const reopened = context.getCompiler().createCompiler();
		reopened.outputFileSystem = compiler.outputFileSystem;
		expect(await run(reopened)).toEqual({ parent: 0, child: 0 });
	}
};
