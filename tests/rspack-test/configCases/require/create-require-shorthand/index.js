import { createRequire } from "module";

it("should keep the key of a shorthand created require", () => {
	const makeRequire = createRequire;
	const load = makeRequire(import.meta.url);
	const o = { load };
	expect(typeof o.load).toBe("function");
});
