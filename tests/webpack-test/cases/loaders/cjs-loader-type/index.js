it("should pass package.json type to loader", function () {
	expect(require("cjs/loader.js!")).toBe("commonjs");
	expect(require("./loader.js!")).toBe("undefined");
});

it("should pass 'commonjs' type to loader for .cjs", function () {
	expect(require("cjs/loader.cjs!")).toBe("commonjs");
	expect(require("./loader.cjs!")).toBe("commonjs");
	// ORIGINAL WEBPACK COMMENT: TODO need fix in v8 https://github.com/nodejs/node/issues/35889
	// ORIGINAL WEBPACK COMMENT: TODO otherwise this test case cause segment fault
	// Turned on this as rspack checks extensions for loader type.
	// So this will not fall into dynamic import which causes segment fault.
	// See: crates/rspack_binding_values/src/plugins/js_loader/resolver.rs
	expect(require("esm/loader.cjs!")).toBe("commonjs");
});
