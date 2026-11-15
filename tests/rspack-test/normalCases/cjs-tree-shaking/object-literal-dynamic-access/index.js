const dynamic = name => name;

it("should keep all properties for a computed access on the require call", () => {
	expect(require("./lib?call")[dynamic("unused")]).toBe("unused");
	expect(Array.isArray(require("./lib?call").usedExports)).toBe(false);
});

it("should keep all properties for a computed access on a require binding", () => {
	const m = require("./lib?binding");
	expect(m.used).toBe("used");
	expect(m[dynamic("unused")]).toBe("unused");
	expect(Array.isArray(m.usedExports)).toBe(false);
});

it("should keep all properties when a require binding escapes", () => {
	const m = require("./lib?escape");
	expect(m.used).toBe("used");
	expect(Object.keys(m)).toEqual(["used", "unused", "usedExports"]);
	expect(Array.isArray(m.usedExports)).toBe(false);
});

it("should keep all properties for a rest element", () => {
	const { used, ...rest } = require("./lib?rest");
	expect(used).toBe("used");
	expect(rest.unused).toBe("unused");
	expect(Array.isArray(rest.usedExports)).toBe(false);
});

it("should keep all properties for a for-in loop", () => {
	const keys = [];
	for (const key in require("./lib?for-in")) keys.push(key);
	expect(keys).toEqual(["used", "unused", "usedExports"]);
});

it("should keep all properties for a computed access through ESM imports", () => {
	const esm = require("./esm");
	expect(esm.readNamespace("unused")).toBe("unused");
	expect(esm.readDefault("unused")).toBe("unused");
	expect(Array.isArray(esm.namespaceUsedExports)).toBe(false);
	expect(Array.isArray(esm.defaultUsedExports)).toBe(false);
});

it("should keep all properties for a computed access on module.exports itself", () => {
	const read = require("./self-computed").read;
	expect(read("unused")).toBe("unused");
	expect(Array.isArray(require("./self-computed").usedExports)).toBe(false);
});

it("should keep all properties when module.exports is aliased", () => {
	const read = require("./self-alias").read;
	expect(read("unused")).toBe("unused");
	expect(Array.isArray(require("./self-alias").usedExports)).toBe(false);
});

it("should not analyze an assignment that is not a top-level statement", () => {
	expect(require("./nested-statement").used).toBe("used");
	expect(Array.isArray(require("./nested-statement").usedExports)).toBe(false);
});
