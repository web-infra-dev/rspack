it("should handle __webpack_module__.id when user declares 'module' variable", () => {
	const moduleId = require("./with-var-collision").default;
	expect(typeof moduleId).toMatch(/^(string|number)$/);
});

it("should handle __webpack_module__.id when user declares 'module' function", () => {
	const moduleId = require("./with-function-collision").default;
	expect(typeof moduleId).toMatch(/^(string|number)$/);
});

it("should handle __webpack_module__.id when user declares 'module' class", () => {
	const moduleId = require("./with-class-collision").default;
	expect(typeof moduleId).toMatch(/^(string|number)$/);
});

it("should handle __webpack_module__ when user declares 'module' variable", () => {
	const mod = require("./with-module-var-collision").default;
	expect(mod.exports).toBeTypeOf("object");
	expect(typeof mod.id).toMatch(/^(string|number)$/);
});
