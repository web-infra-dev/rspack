import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const outputPath = config.output.path;
		const read = file => readFileSync(path.join(outputPath, file), "utf-8");
		const modules = readdirSync(outputPath).filter(file => file.endsWith(".mjs"));
		expect(modules).toHaveLength(2);

		for (const [page, baseURL] of [
			["absolute.html", "https://cdn.example.test/assets/"],
			["relative.html", "http://page.example.test/site/assets/"],
			["root-relative.html", "http://page.example.test/assets/"],
			["scheme-relative.html", "http://cdn.example.test:443/assets/"]
		]) {
			const pageURL = new URL(page, "http://page.example.test/site/pages/");
			const html = read(page);
			const maps = [
				...html.matchAll(
					/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/g
				)
			];
			expect(maps).toHaveLength(1);
			expect(maps[0].index).toBeLessThan(html.indexOf("<base"));
			const map = JSON.parse(maps[0][1]);
			expect(map.imports).toEqual({ external: "./external.mjs" });
			expect(map.scopes).toEqual({ "./scope/": { external: "./scoped.mjs" } });

			const integrity = Object.fromEntries(
				Object.entries(map.integrity).map(([url, value]) => [
					new URL(url, pageURL).href,
					value
				])
			);
			expect(integrity).toEqual({
				[new URL("./external.mjs", pageURL).href]: "sha384-external",
				...Object.fromEntries(
					modules.flatMap(file => {
						const hash = createHash("sha384").update(read(file)).digest("base64");
						return [pageURL, baseURL].map(base => [
							new URL(file, base).href,
							`sha384-${hash}`
						]);
					})
				)
			});
		}
	}
};
