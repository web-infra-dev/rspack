import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		const read = file => readFileSync(path.join(outputPath, file), "utf-8");
		const hashes = ["main+entry.mjs", "assets/bundle.mjs", "bundle.mjs"].map(file => [
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
			["inline-before-base.html", "http://page.example.test/site/pages/assets/"],
			["absolute-deferred-before-base.html", "https://cdn.example.test/assets/"],
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
			expect(generated.index).toBeLessThan(html.indexOf('src="./main%2Bentry.mjs"'));
			if (html.includes("modulepreload")) {
				expect(generated.index).toBeGreaterThan(html.indexOf("modulepreload"));
				expect(generated[0]).toContain("preload-nonce");
			}
			const map = JSON.parse(generated[1]);
			expect(resolveIntegrity(map, baseURL)).toEqual(integrityFor(baseURL));
			// If CSP rejects the base, the same relative keys use the page URL.
			expect(resolveIntegrity(map, pageURL)).toEqual(integrityFor(pageURL));
		}

		const sameBasename = read("same-basename-before-base.html");
		const sameBasenameMaps = maps(sameBasename);
		expect(sameBasenameMaps).toHaveLength(2);
		expect(sameBasenameMaps[1].index).toBeGreaterThan(sameBasename.indexOf("<base"));
		expect(sameBasenameMaps[1].index).toBeLessThan(sameBasename.indexOf('src="/app/main%2Bentry.mjs"'));
		expect(resolveIntegrity(JSON.parse(sameBasenameMaps[1][1]), "https://cdn.example.test/assets/")).toEqual(
			integrityFor("https://cdn.example.test/app/")
		);

		const early = read("base-after-module.html");
		const earlyMaps = maps(early);
		const pageURL = "http://page.example.test/site/pages/base-after-module.html";
		expect(earlyMaps).toHaveLength(1);
		expect(earlyMaps[0].index).toBeLessThan(early.indexOf("<base"));
		expect(resolveIntegrity(JSON.parse(earlyMaps[0][1]), pageURL)).toEqual(
			integrityFor(pageURL)
		);

		const deferred = read("deferred-import-before-map.html");
		const deferredMaps = maps(deferred);
		expect(deferredMaps).toHaveLength(1);
		const inlineModule = deferred.indexOf('id="deferred-module"');
		expect(inlineModule).toBeGreaterThanOrEqual(0);
		expect(deferredMaps[0].index).toBeGreaterThan(inlineModule);
		expect(deferredMaps[0].index).toBeLessThan(deferred.indexOf('src="./main%2Bentry.mjs"'));
		expect(resolveIntegrity(JSON.parse(deferredMaps[0][1]), pageURL)).toEqual(integrityFor(pageURL));

		const manualMaps = maps(read("deferred-import-before-two-maps.html"));
		expect(manualMaps).toHaveLength(3);
		expect(JSON.parse(manualMaps[0][1])).toEqual({ imports: { one: "./one.mjs" } });
		expect(JSON.parse(manualMaps[1][1])).toEqual({ imports: { two: "./two.mjs" } });
		expect(manualMaps[2][0]).toContain("author-nonce");
		expect(resolveIntegrity(JSON.parse(manualMaps[2][1]), pageURL)).toEqual(integrityFor(pageURL));

		const absoluteDeferred = read("absolute-deferred-without-injection.html");
		const absoluteDeferredMaps = maps(absoluteDeferred);
		const lateBase = "https://cdn.example.test/assets/";
		expect(absoluteDeferredMaps).toHaveLength(2);
		expect(JSON.parse(absoluteDeferredMaps[0][1])).toEqual({ imports: { external: "./external.mjs" } });
		expect(absoluteDeferredMaps[1].index).toBeGreaterThan(absoluteDeferred.indexOf("<base"));
		expect(resolveIntegrity(JSON.parse(absoluteDeferredMaps[1][1]), lateBase)).toEqual(integrityFor(lateBase));

		const queryModule = read("different-query-module.html");
		const queryModuleMaps = maps(queryModule);
		expect(queryModuleMaps).toHaveLength(1);
		expect(queryModuleMaps[0].index).toBeGreaterThan(queryModule.indexOf("?variant=vendor"));
		expect(queryModuleMaps[0].index).toBeLessThan(queryModule.lastIndexOf('src="./main%2Bentry.mjs?'));
		expect(resolveIntegrity(JSON.parse(queryModuleMaps[0][1]), pageURL)).toEqual(integrityFor(pageURL));

		const rewritten = read("rewritten-module-src.html");
		const rewrittenMaps = maps(rewritten);
		expect(rewrittenMaps).toHaveLength(1);
		expect(rewritten).toContain('src="/served/main%2Bentry.mjs"');
		expect(rewrittenMaps[0].index).toBeLessThan(rewritten.indexOf('src="/served/'));
		expect(resolveIntegrity(JSON.parse(rewrittenMaps[0][1]), pageURL)).toEqual(
			integrityFor("http://page.example.test/served/")
		);

		const withQuery = read("different-query-preload.html");
		const queryMaps = maps(withQuery);
		expect(queryMaps).toHaveLength(1);
		expect(queryMaps[0].index).toBeGreaterThan(withQuery.indexOf("modulepreload"));
		expect(queryMaps[0].index).toBeLessThan(withQuery.indexOf('src="./main%2Bentry.mjs?'));
		expect(resolveIntegrity(JSON.parse(queryMaps[0][1]), pageURL)).toEqual(integrityFor(pageURL));

		const authored = JSON.parse(maps(read("authored.html"))[0][1]);
		const authoredURL = "http://page.example.test/authored.html";
		expect(resolveIntegrity(authored, authoredURL)).toEqual({
			...integrityFor(authoredURL),
			"http://page.example.test/bundle.mjs": "sha384-authored"
		});

		for (const [file, baseURL] of [
			["multiple-authored.html", "https://page.example.test/"],
			["multiple-authored-other-port.html", "https://cdn.example.test:8443/app/"],
			["multiple-authored-relative-ports.html", "https://cdn.example.test/app/"]
		]) {
			const multiple = read(file);
			const multipleMaps = maps(multiple);
			expect(multipleMaps).toHaveLength(3);
			expect(multipleMaps[0].index).toBeLessThan(multiple.indexOf("modulepreload"));
			expect(multipleMaps[1].index).toBeGreaterThan(multiple.indexOf("modulepreload"));
			expect(JSON.parse(multipleMaps[0][1])).toEqual({ imports: { external: "./external.mjs" } });
			expect(JSON.parse(multipleMaps[1][1])).toEqual({ integrity: { "./bundle.mjs": "sha384-authored" } });
			expect(multipleMaps[2][0]).toContain("authored-nonce");
			expect(multipleMaps[2].index).toBeLessThan(multiple.search(/<script\b[^>]*type=["']?module\b/));
			expect(resolveIntegrity(JSON.parse(multipleMaps[2][1]), "https://page.example.test/" + file)).toEqual(
				integrityFor(baseURL)
			);
		}
	}
};
