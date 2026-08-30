it("should keep all exports when an exported function uses this", () => {
	const m = require("./module");
	expect(m.buildCanonicalString("/bucket")).toBe("resource:/bucket");
	expect(m.usedExports).toBe(true);
});

it("should keep all exports for module.exports and top-level this members", () => {
	const a = require("./module-module-exports");
	const b = require("./module-top-this");
	expect(a.a()).toBe("b");
	expect(b.a()).toBe("b");
	expect(a.usedExports).toBe(true);
	expect(b.usedExports).toBe(true);
});

it("should keep all exports when this is captured by an arrow", () => {
	const m = require("./module-arrow");
	expect(m.a()).toBe("b");
	expect(m.usedExports).toBe(true);
});

it("should keep all exports for defineProperty values and accessors", () => {
	const m = require("./module-define");
	expect(m.a()).toBe("b");
	expect(m.c).toBe("b");
	m.d = "!";
	expect(m.received).toBe("b!");
	expect(m.usedExports).toBe(true);
});

it("should detect this in generator, async, default-param and computed accesses", async () => {
	const m = require("./module-misc");
	expect(m.viaGenerator().next().value).toBe("h");
	expect(await m.viaAsync()).toBe("h");
	expect(m.viaDefaultParam()).toBe("h");
	expect(m.viaComputed()).toBe("h");
	expect(m.usedExports).toBe(true);
});

it("should keep all exports when consumed by ESM", () => {
	expect(require("./esm").result).toBe("resource:/esm");
});

it("should ignore this in nested functions and exported classes", () => {
	const inner = require("./module-inner");
	const klass = require("./module-class");
	expect(inner.a()).toBe(2);
	expect(inner.usedExports).toEqual(["a", "usedExports"]);
	expect(new klass.Impl().x).toBe("x");
	expect(klass.usedExports).toEqual(["Impl", "usedExports"]);
});

it("should ignore this as a word or outside an exported value", () => {
	const word = require("./module-this-word");
	const outside = require("./module-no-this");
	expect(word.a("x")).toBe("string16");
	expect(word.usedExports).toEqual(["a", "usedExports"]);
	expect(outside.a()).toBe("function");
	expect(outside.usedExports).toEqual(["a", "usedExports"]);
});
