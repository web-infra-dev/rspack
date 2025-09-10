export const a = 42;

import b from "./b.js";

it("should load c.js", () => {
	expect(b).toBe("foo");

	const d = require("./d.js");

	expect(d.default).toEqual({});
});

export { b }

export const c = require("./c.js");
