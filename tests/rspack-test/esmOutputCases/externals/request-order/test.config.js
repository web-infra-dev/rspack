const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { execFileSync } = require("node:child_process");

module.exports = {
	findBundle() { return []; },
	snapshotFileFilter(file) { return file.endsWith("/named.mjs"); },
	afterExecute(configs) {
		for (const options of configs) {
			fs.cpSync(path.join(__dirname, "fixtures"), path.join(options.output.path, "fixtures"), { recursive: true });
			const prefix = options.output.filename.replace("[name].mjs", "");
			for (const entry of Object.keys(options.entry)) {
				const url = pathToFileURL(path.join(options.output.path, prefix, `${entry}.mjs`)).href;
				const result = JSON.parse(execFileSync(process.execPath, ["--input-type=module", "-e", `
					const mod = await import(${JSON.stringify(url)});
					const before = mod.a;
					if (mod.setA) mod.setA("updated");
					console.log(JSON.stringify({ order: globalThis.order, before, after: mod.a }));
				`], { encoding: "utf8" }));
				expect(result.order).toEqual(entry === "reverse" ? ["c", "b", "a"] : entry === "duplicate" ? ["a", "c", "b"] : ["a", "b", "c"]);
				if (entry === "named") {
					expect(result.before).toBe("a");
					expect(result.after).toBe("updated");
				}
			}
		}
	}
};
