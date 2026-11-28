import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { promisify } from "node:util";

export default {
	moduleScope(scope, stats, options) {
		scope.rebuildInSameCompiler = async () => {
			const compiler = options.plugins.find(
				plugin => plugin.name === "ModuleOwnedDependenciesTest"
			).compiler;
			const updates = options.plugins.find(
				plugin => plugin.constructor.name === "HotUpdatePlugin"
			);
			const filename = path.join(options.output.path, "main.js");
			for (let iteration = 1; iteration < 5; iteration++) {
				// Repeat the initial build, switch to b, edit its async module, then switch back.
				if (iteration > 1) await updates.goNext();
				const stats = await promisify(compiler.run.bind(compiler))();
				expect(stats.hasErrors()).toBe(false);
				const output = { exports: {} };
				new Function("require", "module", "exports", "it", fs.readFileSync(filename, "utf8"))(
					createRequire(filename), output, output.exports, () => {}
				);
				const selected = iteration === 2 || iteration === 3 ? "b" : "a";
				expect(output.exports.sync).toBe(selected);
				expect(await output.exports.async()).toBe(iteration === 3 ? "updated" : selected);
			}
		};
		return scope;
	}
};
