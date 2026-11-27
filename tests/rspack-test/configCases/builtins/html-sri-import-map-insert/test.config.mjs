import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		const html = readFileSync(path.join(outputPath, "pages/index.html"), "utf-8");
		const mapScript = html.match(
			/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/
		);
		expect(mapScript).toBeTruthy();
		expect(mapScript[0]).toContain("test-nonce");
		expect(mapScript.index).toBeLessThan(html.indexOf("modulepreload"));

		for (const [page, nonce] of [
			["pages/nonce.html", "module-nonce"],
			["pages/nonce-unrelated-preload.html", "module-nonce"],
			["pages/nonce-bundled-preload.html", "preload-nonce"]
		]) {
			const html = readFileSync(path.join(outputPath, page), "utf-8");
			const nonceMap = html.match(/<script\b[^>]*type=["']?importmap["']?[^>]*>/);
			expect(nonceMap).toBeTruthy();
			expect(nonceMap[0].match(/\bnonce=["']?([^"'\s>]+)/)?.[1]).toBe(nonce);
			expect(nonceMap.index).toBeLessThan(html.indexOf("modulepreload"));
		}

		const chunks = readdirSync(outputPath).filter(
			file => file.endsWith(".mjs") && file !== "main.mjs"
		);
		expect(chunks).toHaveLength(1);
		const [chunk] = chunks;
		const content = readFileSync(path.join(outputPath, chunk));
		expect(JSON.parse(mapScript[1]).integrity[`../${chunk}`]).toBe(
			`sha384-${createHash("sha384").update(content).digest("base64")}`
		);
	}
};
