const { promisify } = require("node:util");

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
module.exports = {
	description:
		"should keep a live child's SST readable when another compiler opens the cache",
	options(context) {
		return {
			experiments: { newCache: true },
			incremental: false,
			cache: {
				type: "persistent",
				maxMemoryGenerations: 0,
				storage: { type: "filesystem", location: context.getDist("cache") }
			},
			infrastructureLogging: { level: "none" }
		};
	},
	async build(context, compiler) {
		const warnings = [];
		let resolveStored;
		const waitForStore = () => new Promise(resolve => (resolveStored = resolve));
		compiler.hooks.infrastructureLog.tap("ChildSstTest", (name, type, args) => {
			if (name !== "rspack.cache.IdleFileCache") return;
			if (type === "warn") warnings.push(args[0]);
			if (type === "log" && args[0].startsWith("Stored cache")) resolveStored?.();
		});

		const run = async target => {
			const stats = await promisify(target.run.bind(target))();
			expect(stats.hasErrors()).toBe(false);
		};
		const close = target => promisify(target.close.bind(target))();
		const children = [];
		let parentCompilation, child, parentStored;
		const createChild = name => {
			const target = parentCompilation.createChildCompiler(
				name,
				{ filename: `${name}.js` },
				[
					new compiler.rspack.EntryPlugin(compiler.context, "./a", { name: "main" })
				]
			);
			children.push(target);
			return target;
		};

		compiler.hooks.make.tapPromise("ChildSstTest", async compilation => {
			parentCompilation = compilation;
			// The parent has already opened the empty database. Seed advances CURRENT.
			const seed = createChild("seed");
			await run(seed);
			await close(seed);
			children.pop();

			// Keep this child's newer SST on disk, but not in the memory cache.
			child = createChild("child");
			const childStored = waitForStore();
			await run(child);
			await childStored;
			parentStored = waitForStore();
		});

		try {
			await run(compiler);
			// With a shared database, the stale parent's commit rolls CURRENT back.
			await parentStored;
			// Opening another child cleans files above CURRENT. No manual deletion.
			await run(createChild("late"));
			// The child must still be able to read its own SST after that open.
			await run(child);
		} finally {
			for (const target of children) await close(target);
			await context.closeCompiler();
		}
		expect(warnings, warnings.join("\n")).toEqual([]);
	}
};
