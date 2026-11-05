import fs from "node:fs";
import { fileURLToPath } from "node:url";

it("should leave the startup alone where top-level await is unavailable", () => {
	const bundle = fs.readFileSync(fileURLToPath(import.meta.url), "utf-8");
	const awaited = bundle.split("\n").filter(line => /^\S*exports\S* = await /.test(line));
	expect(awaited).toEqual([]);
});

await Promise.resolve();
