import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		expect(readdirSync(path.join(outputPath, "chunks+v1"))).toContain("other.mjs");
		expect(readdirSync(outputPath)).toContain("other-lazy.mjs");
		const digest = file => {
			const content = readFileSync(path.join(outputPath, file));
			return `sha384-${createHash("sha384").update(content).digest("base64")}`;
		};

		for (const [page, files] of [
			[
				"index.html",
				["app.mjs", "runtime.mjs", "shared.mjs", "vendor.mjs"]
			],
			["sync.html", ["sync.mjs", "runtime.mjs", "shared.mjs"]]
		]) {
			const html = readFileSync(path.join(outputPath, page), "utf-8");
			expect(html).toContain(`src="/chunks%2Bv1/${files[0]}"`);
			const map = html.match(
				/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/
			);
			expect(map).toBeTruthy();

			const expectedIntegrity = Object.fromEntries(files.flatMap(file => {
				const hash = digest(`chunks+v1/${file}`);
				return [[`/chunks+v1/${file}`, hash], [`/chunks%2Bv1/${file}`, hash]];
			}));
			if (page === "index.html") {
				expectedIntegrity["/vendor-lazy.mjs"] = digest("vendor-lazy.mjs");
			}
			expect(JSON.parse(map[1]).integrity).toEqual(expectedIntegrity);
		}

		const collision = readFileSync(path.join(outputPath, "encoded-collision.html"), "utf-8");
		expect(collision).toContain('src="/chunks%2Bv1/sync.mjs"');
		expect(collision).toContain('src="/chunks%252Bv1/sync.mjs"');
		const map = collision.match(/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/);
		expect(map).toBeTruthy();
		const { integrity } = JSON.parse(map[1]);
		expect(integrity["/chunks%2Bv1/shared.mjs"]).toBe(digest("chunks+v1/shared.mjs"));
		expect(integrity["/chunks%252Bv1/shared.mjs"]).toBe(digest("chunks%2Bv1/shared.mjs"));
	}
};
