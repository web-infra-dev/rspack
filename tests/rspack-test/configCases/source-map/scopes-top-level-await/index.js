import { value as asyncAlias } from "./value";

const fs = require("fs");
const result = Promise.resolve(asyncAlias + 1);

it("keeps module scopes inside the generated async wrapper", async () => {
	expect(await result).toBe(22);

	const map = JSON.parse(fs.readFileSync(__filename + ".map", "utf8"));
	const source = map.sources.findIndex(name => name.endsWith("/index.js"));
	expect(map.scopes[source]).toEqual(expect.any(String));
	expect(map.ranges).toHaveLength(1);
	expect(map.names).toContain("asyncAlias");
	expect(map.names).toContain("result");
});

await result;
