import fs from "node:fs";
import path from "node:path";
import { promisify } from "node:util";

export default {
	moduleScope(scope, stats, options) {
		scope.rebuildIssuers = async () => {
			const plugin = options.plugins.find(
				plugin => plugin.name === "CaptureSharedIssuer"
			);
			const compiler = plugin.compiler;
			const entry = path.join(options.context, "index.js");
			const unrelated = path.join(options.context, "unrelated.js");
			const initialIssuer = path.basename(plugin.issuerSnapshots[0]);
			// Either importer can win the initial parallel build. Remove that one.
			const updatedEntry = fs.readFileSync(entry, "utf8").replace(
				`import "./${initialIssuer}";\n`, ""
			);
			for (const [file, content] of [
				[entry, updatedEntry],
				[unrelated, 'globalThis.unrelated = "first edit";\n'],
				[unrelated, 'globalThis.unrelated = "second edit";\n']
			]) {
				fs.writeFileSync(file, content);
				plugin.modifiedFiles = new Set([file]);
				const stats = await promisify(compiler.run.bind(compiler))();
				expect(stats.hasErrors()).toBe(false);
			}
		};
		return scope;
	}
};
