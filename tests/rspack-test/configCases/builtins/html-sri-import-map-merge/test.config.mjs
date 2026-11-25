import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		const html = readFileSync(path.join(outputPath, "index.html"), "utf-8");
		const mapScripts = [
			...html.matchAll(
				/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/g
			)
		];
		expect(mapScripts).toHaveLength(1);
		expect(mapScripts[0][0]).toContain("test-nonce");
		expect(mapScripts[0].index).toBeLessThan(html.indexOf("modulepreload"));

		const map = JSON.parse(mapScripts[0][1]);
		expect(map.imports).toEqual({ external: "/vendor/external.mjs" });
		expect(map.scopes).toEqual({ "/vendor/": { shared: "/vendor/shared.mjs" } });
		expect(map.integrity["/vendor/external.mjs"]).toBe("sha384-external");

		const chunksPath = path.join(outputPath, "chunks");
		const chunks = readdirSync(chunksPath);
		expect(chunks).toHaveLength(2);
		for (const chunk of chunks) {
			const url = `https://cdn.example.test/assets/chunks/${chunk}`;
			const content = readFileSync(path.join(chunksPath, chunk));
			expect(map.integrity[url]).toBe(
				`sha384-${createHash("sha384").update(content).digest("base64")}`
			);
		}

		expect(Object.keys(map.integrity).some(url => url.endsWith("/other.mjs"))).toBe(
			false
		);
		const otherHtml = readFileSync(path.join(outputPath, "other.html"), "utf-8");
		expect(otherHtml).not.toContain("importmap");

		const manualHtml = readFileSync(path.join(outputPath, "manual-other.html"), "utf-8");
		const manualMap = manualHtml.match(/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/);
		expect(manualMap).not.toBeNull();
		const other = readFileSync(path.join(outputPath, "entries/other.mjs"));
		expect(JSON.parse(manualMap[1]).integrity).toEqual({
			"https://cdn.example.test/assets/entries/other.mjs":
				"sha384-" + createHash("sha384").update(other).digest("base64")
		});
	}
};
