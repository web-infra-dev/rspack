it("should keep the key of a shorthand `require`", () => {
	const o = { used: "used", require };
	expect(o.used).toBe("used");
	expect(typeof o.require).toBe("function");
});

it("should keep the key of shorthand `__filename` and `__dirname`", () => {
	const o = { used: "used", __filename, __dirname };
	expect(o).toEqual({ used: "used", __filename: "/index.js", __dirname: "/" });
});

it("should keep the key of shorthand `global` and `define`", () => {
	const o = { global, define };
	expect(typeof o.global).toBe("object");
	expect(typeof o.define).toBe("function");
});

it("should keep the key of shorthand api identifiers", () => {
	const o = {
		__webpack_public_path__,
		__webpack_module__,
		__non_webpack_require__,
		__webpack_layer__,
		__webpack_exports_info__
	};
	expect(typeof o.__webpack_public_path__).toBe("string");
	expect(o.__webpack_module__.exports).toBe(module.exports);
	expect(typeof o.__non_webpack_require__).toBe("function");
	expect(o.__webpack_layer__).toBe(null);
	expect(o.__webpack_exports_info__).toBe(true);
});
