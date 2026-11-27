import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

export default {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const read = file => readFileSync(path.join(config.output.path, file), "utf-8");
		const html = read("index.html");
		const map = html.match(/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/);
		expect(map).toBeTruthy();
		expect(read("app.mjs")).toMatch(/from ["']\.\/runtime\.mjs["']/);
		expect(read("runtime.mjs")).toContain("__import__(");
		expect(read("async.mjs")).toContain("42");
		const expected = Object.fromEntries(["app.mjs", "runtime.mjs", "vendor.mjs"].map(file => [
			`/${file}`,
			`sha384-${createHash("sha384").update(read(file)).digest("base64")}`
		]));
		expect(JSON.parse(map[1]).integrity).toEqual(expected);
	}
};
