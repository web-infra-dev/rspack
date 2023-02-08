it("modules-true should use css modules", () => {
	const css = require("./modules-true.css");
	expect(css).toEqual({ "module-true": "_module-true_3vgpi_1" });
});

it("modules-false should not use css modules", () => {
	const css = require("./modules-false.module.css");
	expect(css).toEqual({});
});

it("auto-true should use css modules", () => {
	const css = require("./auto-true.module.css");
	expect(css).toEqual({ "auto-true": "_auto-true_mtxdv_1" });
});

it("auto-false should not use css modules", () => {
	const css = require("./auto-false.module.css");
	expect(css).toEqual({});
});

it("auto-regex should use css modules", () => {
	const css = require("./auto-regex.css");
	expect(css).toEqual({ "auto-regex": "_auto-regex_1j4lx_1" });
});

it("auto-function should use css modules", () => {
	const css = require("./auto-function.css");
	expect(css).toEqual({ "auto-function": "_auto-function_b0amx_1" });
});

it("generateScopedName should use right name for css modules", () => {
	const css = require("./generateScopedName.module.css");
	expect(css).toEqual({
		"generate-scoped-name":
			"generateScopedName-module__generate-scoped-name___xw790"
	});
});

it("localsConvention should use right convention for css modules", () => {
	const css = require("./localsConvention.module.css");
	expect(css).toEqual({
		"locals-convention": "_locals-convention_1eiju_1",
		localsConvention: "_locals-convention_1eiju_1"
	});
});
