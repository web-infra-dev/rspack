import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		const read = file => readFileSync(path.join(outputPath, file), "utf-8");
		const hashes = ["assets/bundle.mjs", "bundle.mjs"].map(file => [
			file,
			`sha384-${createHash("sha384").update(read(file)).digest("base64")}`
		]);
		const maps = html => [
			...html.matchAll(
				/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/g
			)
		];
		const resolveIntegrity = (map, pageURL) =>
			Object.fromEntries(
				Object.entries(map.integrity).map(([url, hash]) => [
					new URL(url, pageURL).href,
					hash
				])
			);

		for (const [page, baseURL, authored = true] of [
			["absolute.html", "https://cdn.example.test/assets/"],
			["relative.html", "http://page.example.test/site/assets/"],
			["root-relative.html", "http://page.example.test/assets/"],
			["scheme-relative.html", "http://cdn.example.test:443/assets/"],
			["collision.html", "http://page.example.test/site/pages/assets/"],
			["normalized-collision.html", "http://page.example.test/site/pages/assets/"],
			["preload-before-base.html", "http://page.example.test/assets/"],
			["preload-before-colliding-base.html", "http://page.example.test/site/pages/assets/"],
			["generated-before-base.html", "http://page.example.test/assets/", false],
			["base-after-module.html", "http://page.example.test/site/pages/", false]
		]) {
			const pageURL = new URL(page, "http://page.example.test/site/pages/");
			const html = read(page);
			const mapScripts = maps(html);
			expect(mapScripts).toHaveLength(1);
			expect(mapScripts[0].index).toBeLessThan(html.indexOf("<base"));
			const map = JSON.parse(mapScripts[0][1]);
			if (authored) {
				expect(map.imports).toEqual({ external: "./external.mjs" });
				expect(map.scopes).toEqual({ "./scope/": { external: "./scoped.mjs" } });
			}
			if (html.includes("modulepreload")) {
				expect(mapScripts[0].index).toBeLessThan(html.indexOf("modulepreload"));
			}
			if (page === "generated-before-base.html") {
				expect(mapScripts[0][0]).toContain("preload-nonce");
			}

			expect(resolveIntegrity(map, pageURL)).toEqual({
				...(authored && {
					[new URL("./external.mjs", pageURL).href]: "sha384-external"
				}),
				...Object.fromEntries(
					[pageURL, baseURL].flatMap(base =>
						hashes.map(([file, hash]) => [new URL(file, base).href, hash])
					)
				)
			});
		}

		const authored = JSON.parse(maps(read("authored.html"))[0][1]);
		expect(resolveIntegrity(authored, "http://page.example.test/authored.html")).toEqual({
			"http://page.example.test/assets/bundle.mjs": hashes[0][1],
			"http://page.example.test/bundle.mjs": "sha384-authored"
		});
	}
};
