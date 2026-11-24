it("should preserve codeFilter and parallel metadata for ??ident inline requests", () => {
	expect(require("!!./transform-loader.mjs??string-filter!./str-hit.js")).toEqual({
		value: "string-hit",
		parallel: true
	});
	expect(require("!!./transform-loader.mjs??string-filter!./str-miss.js")).toEqual({
		original: "str-miss"
	});
	expect(require("!!./transform-loader.mjs??regexp-filter!./reg-hit.js")).toEqual({
		value: "regexp-hit",
		parallel: true
	});
	expect(require("!!./transform-loader.mjs??regexp-filter!./reg-miss.js")).toEqual({
		original: "reg-miss"
	});
});

it("should preserve codeFilter and parallel metadata across importModule remainingRequest", () => {
	expect(require("./pitch-hit.virtual.js")).toEqual({
		value: "pitched-hit",
		parallel: true
	});
	expect(require("./pitch-miss.virtual.js")).toEqual({
		original: "pitch-miss"
	});
});
