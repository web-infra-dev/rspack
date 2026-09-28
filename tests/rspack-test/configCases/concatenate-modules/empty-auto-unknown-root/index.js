import { getMissing } from "./consumer";

const barrel = require("./barrel");

it("does not turn an unknown root export into an own undefined property", () => {
	expect(getMissing()).toBeUndefined();
	expect(barrel.JsTyping.value).toBe(42);
	const compiler = __STATS__.modules.find(module => module.name === "./compiler.js");
	expect(compiler).toBeDefined();
	expect(compiler.providedExports).toBe(null);
	expect(compiler.optimizationBailout).toEqual(
		expect.arrayContaining([expect.stringContaining("List of module exports is dynamic")])
	);
});
