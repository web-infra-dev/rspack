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

it("should preserve alias query options when rule attaches metadata-only ident", () => {
	expect(require("./alias-hit.alias-filter.js")).toEqual({
		value: "from-alias",
		parallel: false
	});
	expect(require("./alias-miss.alias-filter.js")).toEqual({
		original: "alias-miss"
	});
	expect(require("./alias-cache.alias-cache.js")).toEqual({
		value: "from-alias",
		parallel: false
	});
	expect(require("./alias-override.alias-override.js")).toEqual({
		value: "from-options",
		parallel: false
	});
});

it("should apply metadata and override alias query for explicit ??ident inline requests", () => {
	expect(
		require("!!aliased-transform-loader??alias-meta-only!./alias-inline-hit.js")
	).toEqual({
		parallel: false
	});
	expect(
		require("!!aliased-transform-loader??alias-meta-only!./alias-inline-miss.js")
	).toEqual({
		original: "alias-inline-miss"
	});
});

it("should keep functional Rule.use object-options and optionless metadata variants distinct", () => {
	expect(require("./func-issuer-options.js")).toEqual({
		value: "from-func-options",
		parallel: false
	});
	expect(require("./func-issuer-meta-only.js")).toEqual({
		parallel: false
	});
	expect(require("./miss.func-shared.js")).toEqual({
		original: "func-miss"
	});
});
