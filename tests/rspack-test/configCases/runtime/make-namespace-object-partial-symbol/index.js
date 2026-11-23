it("should preserve the undefined export without Symbol.toStringTag", () => {
	expect(typeof Symbol).toBe("function");
	expect(Symbol.toStringTag).toBeUndefined();

	const ns = require("./esm");
	expect(ns.__esModule).toBe(true);
	expect(ns.undefined).toBe(1);
	expect(Object.keys(ns)).toEqual(["undefined"]);
});
