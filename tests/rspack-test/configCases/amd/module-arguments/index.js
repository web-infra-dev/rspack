it("should pass the module's exports in a constant dependency array", () => {
	const lib = require("./define-constant");
	expect(lib.value).toBe(42);
	expect(lib.self).toBe(lib);
});

it("should retain the exports parameter for a constant AMD require array", async () => {
	const lib = require("./require-constant");
	expect(await lib.load()).toBe(lib);
});

it("should preserve the exports argument for a constant AMD require array in an ES module", async () => {
	const lib = require("./require-esm");
	expect(await lib.loadConstant()).toBe(lib);
});

it("should pass the module and runtime require to AMD require", async () => {
	const lib = require("./require-module");
	expect((await lib.load()).exports).toBe(lib);
	const [loader, mod, exports] = await lib.loadMixed();
	expect(loader).toBe(__webpack_require__);
	expect(mod.exports).toBe(lib);
	expect(exports).toBe(lib);
});

it("should retain the module parameter for a constant AMD require array", async () => {
	const lib = require("./require-module-constant");
	expect((await lib.load()).exports).toBe(lib);
	const [loader, mod, exports] = await lib.loadMixed();
	expect(loader).toBe(__webpack_require__);
	expect(mod.exports).toBe(lib);
	expect(exports).toBe(lib);
});

it("should preserve the module argument when a local variable shadows module", async () => {
	const lib = require("./require-module-shadowed");
	const mod = await lib.load();
	expect(mod.exports).toBe(lib);
	expect(await lib.loadConstant()).toBe(mod);
	expect(mod).not.toBe(lib.shadow);
	expect(lib.shadow.value).toBe(42);
});

it("should pass the module argument to define when a local variable shadows module", () => {
	const lib = require("./define-module-shadowed");
	expect(lib.mod.exports).toBe(lib);
	expect(lib.shadow.value).toBe("local");
});

it("should assign the factory result to the module argument when module is shadowed", () => {
	const lib = require("./define-factory-shadowed");
	expect(lib.value).toBe(42);
	expect(lib.mod.exports).toBe(lib);
	expect(lib.shadow.value).toBe("local");
});

it("should assign the defined object to the module argument when module is shadowed", () => {
	const lib = require("./define-object-shadowed");
	expect(lib.value).toBe(42);
	expect(typeof lib.shadow).toBe("function");
});
