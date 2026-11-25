import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		expect(readdirSync(outputPath)).toEqual(
			expect.arrayContaining(["other.mjs", "other-lazy.mjs"])
		);

		for (const [page, files] of [
			[
				"index.html",
				["app.mjs", "runtime.mjs", "shared.mjs", "vendor.mjs", "vendor-lazy.mjs"]
			],
			["sync.html", ["sync.mjs", "runtime.mjs", "shared.mjs"]]
		]) {
			const html = readFileSync(path.join(outputPath, page), "utf-8");
			const map = html.match(
				/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/
			);
			expect(map).toBeTruthy();

			const expectedIntegrity = Object.fromEntries(
				files.map(file => {
					const content = readFileSync(path.join(outputPath, file));
					const hash = createHash("sha384").update(content).digest("base64");
					return [`/${file}`, `sha384-${hash}`];
				})
			);
			expect(JSON.parse(map[1]).integrity).toEqual(expectedIntegrity);
		}
	}
};
