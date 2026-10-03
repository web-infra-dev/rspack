import { getMissing } from "./consumer";

const barrel = require("./barrel");

it("does not turn an unknown root export into an own undefined property", () => {
	expect(getMissing()).toBeUndefined();
	expect(barrel.JsTyping.value).toBe(42);
	expect(Object.hasOwn(require("./compiler"), "JsTyping")).toBe(false);
	const compiler = __STATS__.modules.find(module =>
		module.name.startsWith("./compiler.js +")
	);
	expect(compiler).toBeDefined();
	expect(compiler.providedExports).toBe(null);
	expect(compiler.modules.some(module => module.name === "./empty.js")).toBe(true);
});
