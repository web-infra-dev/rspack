let value = require("./module.js");
import { a } from "./lib/a.js";

it("should compile", async () => {
	expect(value).toBe(1);
	expect(a).toBe(1);
	await NEXT_HMR();
	value = require("./module");
	expect(value).toBe(2);
});

module.hot.accept("./module.js");
