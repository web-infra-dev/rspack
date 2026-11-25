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

		const nonceHtml = readFileSync(
			path.join(outputPath, "pages/nonce.html"),
			"utf-8"
		);
		const nonceMap = nonceHtml.match(/<script\b[^>]*type=["']?importmap["']?[^>]*>/);
		expect(nonceMap).toBeTruthy();
		expect(nonceMap[0]).toMatch(/\bnonce=["']?module-nonce["']?(?=[\s>])/);
		expect(nonceMap.index).toBeLessThan(nonceHtml.indexOf("modulepreload"));

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
