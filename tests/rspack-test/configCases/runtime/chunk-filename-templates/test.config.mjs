import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
	afterExecute(configs) {
		for (const { output } of configs) {
			const source = fs.readFileSync(path.join(output.path, output.filename), "utf8");
			const normalized = source.replace(/\.[a-f0-9]+\.(js|css)/g, ".HASH.$1");
			expect(normalized).toContain(output.environment.templateLiteral
				? '(chunkId) => (`chunk-${chunkId}.HASH.js?\\`\\${literal}`)'
				: '(chunkId) => ("chunk-" + chunkId + ".HASH.js?`${literal}")');
			expect(normalized).toContain('(chunkId) => (chunkId + ".HASH.css")');
		}
	}
};
