it("should add warning on using as expression", () => {
	let _require1 = require;
	expect(typeof _require1).toBe("function");
	let _require2 = () => require;
	expect(_require2.toString()).toBe("() => /* #__PURE__ */ __webpack_require__(/*! . */ \"./warnings/require-as-expression sync recursive\")")
});
