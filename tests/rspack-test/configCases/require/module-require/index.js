import { createRequire as _createRequire } from "module";
import { createRequire as __createRequire, builtinModules } from "module";
import { createRequire as nodeCreateRequire } from "node:module";
import * as esm from "./esm.mjs";
import { unusedBranchEnabled } from "./flag.js";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

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
	expect(new require("./a")).toEqual({});
	expect(_createRequire(import.meta.url)("./c")).toBe(3);
	_createRequire(import.meta.url)("./c");
	const __rspack_create_require = request => request;
	expect(__rspack_create_require("./a")).toBe("./a");
	var varRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	expect(varRequire("./a")).toBe(4);
	expect(new varRequire("./a")).toEqual({});
});

it("should resolve using created require", () => {
	const require = _createRequire(import.meta.url);
	expect(require.resolve("./b")).toBe("./b.js");
	expect(_createRequire(import.meta.url).resolve("./b")).toBe("./b.js");
});

it("should preserve optional created require members", () => {
	const require = _createRequire(import.meta.url);
	expect(require?.resolve("./b")).toMatch(/[\\/]b\.js$/);
	expect(require?.cache).toBe(_createRequire(import.meta.url).cache);
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

it("should drop inactive branch dependencies of created require", () => {
	const require = _createRequire(import.meta.url);
	if (unusedBranchEnabled) {
		require("./guarded-unused.js");
	}

	const unusedMarker = "__rspackCreateRequire" + "GuardedUnused";
	const emittedSource = fs
		.readdirSync(path.dirname(__filename))
		.filter(file => file.endsWith(".js"))
		.map(file => fs.readFileSync(path.join(path.dirname(__filename), file), "utf-8"))
		.join("\n");
	expect(globalThis[unusedMarker]).toBe(undefined);
	expect(emittedSource.includes(unusedMarker)).toBe(false);
});

it("should not parse relative createRequire filename", () => {
	expect(() => _createRequire("./foo/c.js")("./a")).toThrow(/absolute path|file URL/);
	expect(() => _createRequire("./foo/c.js").resolve("./a")).toThrow(/absolute path|file URL/);
});

it("should not decode encoded separators in createRequire file URLs", () => {
	expect(() => _createRequire("file:///project/foo%2Fbar.js")("./a")).toThrow();
	expect(() => _createRequire("file:///project/foo%5Cbar.js")("./a")).toThrow();
});

it("should preserve createRequire binding for unsupported uses", () => {
	const createRequire = _createRequire;
	const require = _createRequire(import.meta.url);
	expect(() => createRequire("./foo/c.js")).toThrow(/absolute path|file URL/);
	expect(() => _createRequire(...import.meta.url)("./a")).toThrow(/absolute path|file URL/);
	expect(() => _createRequire(import.meta.url)(..."./a")).toThrow();
	expect(() => _createRequire(import.meta.url).resolve()).toThrow();
	expect(
		(function () { return require.resolve(..."./b"); }).toString()
	).toContain("...");
});

it("should not hoist var createRequire bindings before initialization", () => {
	expect(() => {
		hoistedRequire("./a");
		var hoistedRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	}).toThrow();
});

it("should not tag lexical createRequire bindings before initialization", () => {
	expect(() => {
		lexicalRequire("./a");
		const lexicalRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
	}).toThrow();

	expect(() => {
		const before = sameDeclarationRequire("./a"), sameDeclarationRequire = _createRequire(new URL("./foo/c.js", import.meta.url));
		return before;
	}).toThrow();

	const initializedRequire = _createRequire(new URL("./foo/c.js", import.meta.url)), after = initializedRequire("./a");
	expect(after).toBe(4);
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

it("should create require in ESM modules", () => {
	expect(esm.required).toBe(1);
	expect(esm.directRequired).toBe(3);
	expect(esm.resolved).toBe("./b.js");
	expect(esm.nodeRequired).toBe(4);
});
