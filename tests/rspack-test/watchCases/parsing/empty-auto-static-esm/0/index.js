import * as namespace from "./empty";

it("promotes an empty auto module while a static ESM edge exists", () => {
	const required = require("./empty");

	expect(Object.keys(namespace)).toEqual([]);
	expect(required.__esModule).toBe(true);
});
