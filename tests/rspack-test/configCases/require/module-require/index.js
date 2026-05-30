import { createRequire as _createRequire } from "module";
import { createRequire as __createRequire, builtinModules } from "module";
import { createRequire as nodeCreateRequire } from "node:module";

it("should evaluate require/createRequire", () => {
	expect(
		(function () { return typeof _createRequire; }).toString()
	).toBe("function () { return 'function'; }");
	expect(
		(function () { if (typeof _createRequire); }).toString()
	).toBe("function () { if (true); }");
	const require = __createRequire(import.meta.url);
	expect(
		(function () { return typeof require; }).toString()
	).toBe("function () { return 'function'; }");
	expect(
		(function () { if (typeof require); }).toString()
	).toBe("function () { if (true); }");
});

it("should create require", () => {
	const require = _createRequire(import.meta.url);
	expect(require("./a")).toBe(1);
	expect(_createRequire(import.meta.url)("./c")).toBe(3);
	var varRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	expect(varRequire("./a")).toBe(4);
});

it("should resolve using created require", () => {
	const require = _createRequire(import.meta.url);
	expect(require.resolve("./b")).toBe("./b.js");
	expect(_createRequire(import.meta.url).resolve("./b")).toBe("./b.js");
});

it("should provide require.cache", () => {
	const _require = _createRequire(import.meta.url);
	expect(require.cache).toBe(_require.cache);
	expect(require.cache).toBe(_createRequire(import.meta.url).cache);
});

it("should provide dependency context", () => {
	const _require = _createRequire(new URL("./foo/c.js", import.meta.url));
	expect(_require("./a")).toBe(4);
	const _require1 = _createRequire(new URL("./foo/", import.meta.url));
	expect(_require1("./c")).toBe(5);
	expect(
		_createRequire(new URL("./foo/?v=1#hash", import.meta.url))("./c")
	).toBe(5);
	expect(_createRequire(new URL("./foo/c.js", import.meta.url))("./a")).toBe(4);
	const nodeRequire = nodeCreateRequire(new URL("./foo/c.js", import.meta.url));
	expect(nodeRequire("./a")).toBe(4);
});

it("should not parse relative createRequire filename", () => {
	expect(() => _createRequire("./foo/c.js")("./a")).toThrow(/absolute path|file URL/);
	expect(() => _createRequire("./foo/c.js").resolve("./a")).toThrow(/absolute path|file URL/);
});

it("should preserve createRequire binding for unsupported uses", () => {
	const createRequire = _createRequire;
	expect(() => createRequire("./foo/c.js")).toThrow(/absolute path|file URL/);
	expect(() => _createRequire(...import.meta.url)("./a")).toThrow(/absolute path|file URL/);
});

it("should stop parsing reassigned created require bindings", () => {
	let mutableRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	mutableRequire = request => request;
	expect(mutableRequire("./a")).toBe("./a");

	let destructuredRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	({ destructuredRequire } = { destructuredRequire: request => request });
	expect(destructuredRequire("./a")).toBe("./a");
});

it("should preserve createRequire results used as values", () => {
	let assignedRequire;
	assignedRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	expect(assignedRequire("./a")).toBe(4);
});

it("should not parse URL object as CommonJS require request", () => {
	expect(() => require(new URL("./a.js", import.meta.url))).toThrow();
});

it("should add warning on using as expression", () => {
	let _require = _createRequire(new URL("./foo/c.js", import.meta.url));
	const a = _require;
	expect(typeof a).toBe("function");
});

it("should add warning on using require.main", () => {
	let _require = _createRequire(new URL("./foo/c.js", import.meta.url));
	expect(_require.main).toBe(undefined);
	expect(_createRequire(import.meta.url).main).toBe(undefined);
	expect(_createRequire(import.meta.url).resolve).toBe(undefined);
});

it("should import Node.js module", () => {
	expect(Array.isArray(builtinModules)).toBe(true);
});
