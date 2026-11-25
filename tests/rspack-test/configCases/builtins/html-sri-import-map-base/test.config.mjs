import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		const read = file => readFileSync(path.join(outputPath, file), "utf-8");
		const hashes = ["main.mjs", "assets/bundle.mjs", "bundle.mjs"].map(file => [
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
		const integrityFor = base =>
			Object.fromEntries(
				hashes.map(([file, hash]) => [new URL(file, base).href, hash])
			);

		for (const [page, baseURL] of [
			["absolute.html", "https://cdn.example.test/assets/"],
			["relative.html", "http://page.example.test/site/assets/"],
			["root-relative.html", "http://page.example.test/assets/"],
			["scheme-relative.html", "http://cdn.example.test:443/assets/"],
			["collision.html", "http://page.example.test/site/pages/assets/"],
			["normalized-collision.html", "http://page.example.test/site/pages/assets/"],
			["preload-before-base.html", "http://page.example.test/assets/"],
			["preload-before-colliding-base.html", "http://page.example.test/site/pages/assets/"],
			["generated-before-base.html", "http://page.example.test/assets/"]
		]) {
			const pageURL = new URL(page, "http://page.example.test/site/pages/");
			const html = read(page);
			const mapScripts = maps(html);
			const hasAuthoredMap = page !== "generated-before-base.html";
			expect(mapScripts).toHaveLength(hasAuthoredMap ? 2 : 1);
			if (hasAuthoredMap) {
				expect(mapScripts[0].index).toBeLessThan(html.indexOf("<base"));
				expect(JSON.parse(mapScripts[0][1])).toEqual({
					imports: { external: "./external.mjs" },
					scopes: { "./scope/": { external: "./scoped.mjs" } },
					integrity: { "./external.mjs": "sha384-external" }
				});
			}

			const generated = mapScripts.at(-1);
			expect(generated.index).toBeGreaterThan(html.indexOf("<base"));
			expect(generated.index).toBeLessThan(html.search(/<script\b[^>]*type=["']?module\b/));
			if (html.includes("modulepreload")) {
				expect(generated.index).toBeGreaterThan(html.indexOf("modulepreload"));
				expect(generated[0]).toContain("preload-nonce");
			}
			const map = JSON.parse(generated[1]);
			expect(resolveIntegrity(map, baseURL)).toEqual(integrityFor(baseURL));
			// If CSP rejects the base, the same relative keys use the page URL.
			expect(resolveIntegrity(map, pageURL)).toEqual(integrityFor(pageURL));
		}

		const early = read("base-after-module.html");
		const earlyMaps = maps(early);
		const pageURL = "http://page.example.test/site/pages/base-after-module.html";
		expect(earlyMaps).toHaveLength(1);
		expect(earlyMaps[0].index).toBeLessThan(early.indexOf("<base"));
		expect(resolveIntegrity(JSON.parse(earlyMaps[0][1]), pageURL)).toEqual(
			integrityFor(pageURL)
		);

		const authored = JSON.parse(maps(read("authored.html"))[0][1]);
		const authoredURL = "http://page.example.test/authored.html";
		expect(resolveIntegrity(authored, authoredURL)).toEqual({
			...integrityFor(authoredURL),
			"http://page.example.test/bundle.mjs": "sha384-authored"
		});

		const multiple = read("multiple-authored.html");
		const multipleMaps = maps(multiple);
		expect(multipleMaps).toHaveLength(3);
		expect(JSON.parse(multipleMaps[0][1])).toEqual({ imports: { external: "./external.mjs" } });
		expect(JSON.parse(multipleMaps[1][1])).toEqual({ integrity: { "./bundle.mjs": "sha384-authored" } });
		expect(multipleMaps[2][0]).toContain("authored-nonce");
		expect(multipleMaps[2].index).toBeLessThan(multiple.search(/<script\b[^>]*type=["']?module\b/));
		const generatedURL = "http://page.example.test/multiple-authored.html";
		expect(resolveIntegrity(JSON.parse(multipleMaps[2][1]), generatedURL)).toEqual(
			integrityFor(generatedURL)
		);
	}
};
