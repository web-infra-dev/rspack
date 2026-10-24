import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
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
		const unknown = { status: "unknown", reason: null };
		const manager = context.getCompiler();
		const events = [];
		compiler.hooks.done.tap("ChildSstTest", () => events.push("done"));
		compiler.hooks.infrastructureLog.tap("ChildSstTest", (name, type, [message]) => {
			if (name === "rspack.cache.IdleFileCache" && type === "log" && message.startsWith("Stored cache (")) {
				events.push(message);
			}
		});
		const run = async (parent, session, childState = unknown) => {
			parent.outputFileSystem = compiler.outputFileSystem;
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
				const { cacheInfo, children } = stats.toJson({ all: false, cacheInfo: true, children: true });
				expect(cacheInfo.persistent).toEqual(unknown);
				expect(children[0].cacheInfo.persistent).toEqual(childState);
				for (const info of [cacheInfo, children[0].cacheInfo]) {
					expect(info.persistentSessionInitialization).toEqual({ status: session, reason: null });
				}
				return built;
			} finally {
				await promisify(parent.close.bind(parent))();
			}
		};

		expect(await run(compiler, "cold")).toEqual({ parent: 1, child: 1 });
		const cache = context.getDist("cache");
		const [, keys] = /^Commit \d+ (\d+) keys/m.exec(fs.readFileSync(path.join(cache, "LOG"), "utf8"));
		expect(events).toEqual(["done", `Stored cache (${keys} items)`]);
		// Reopening the same directory must recover both compilers' data from disk.
		expect(await run(manager.createCompiler(), "valid")).toEqual({ parent: 0, child: 0 });

		// Damage the data table (family 0) while leaving the validator table intact.
		const tables = fs.readFileSync(path.join(cache, "LOG"), "utf8");
		const [, id] = /^\s*0\s*\|\s*\d+\s*\|\s*(\d+) SST/m.exec(tables);
		const file = path.join(cache, `${id}.sst`);
		fs.writeFileSync(file, Buffer.alloc(fs.statSync(file).size, 0xa5));
		// Only the child has a module to read from the damaged table.
		manager.mergeOptions({ entry: {} });
		const damaged = await run(manager.createCompiler(), "valid", { status: "error", reason: "recovery" });
		expect(damaged).toEqual({ parent: 0, child: 1 });
	}
};
