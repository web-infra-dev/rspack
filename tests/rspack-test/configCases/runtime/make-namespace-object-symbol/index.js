const fs = require("fs");
// `require`ing an ESM module reads `__esModule`, which is what emits
// `__webpack_require__.r`.
const ns = require("./esm");

it("should mark the namespace object", () => {
	expect(ns.__esModule).toBe(true);
	expect(ns[Symbol.toStringTag]).toBe("Module");
	expect(ns.value).toBe(1);
	expect(ns.default).toBe(2);
});

it("should only feature-test Symbol when environment.symbol is false", () => {
	const source = fs.readFileSync(__filename, "utf-8");
	// strings split to not match this test's own source
	expect(source.includes("typeof Sym" + "bol !== 'undefined'")).toBe(
		SHAPE === "feature-test"
	);
	expect(source.includes("Object.define" + "Properties(exports, {")).toBe(
		SHAPE === "define-properties"
	);
});
