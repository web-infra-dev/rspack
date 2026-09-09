it("keeps ordinary hoisted function bindings", () => {
	expect(require("./missing-hoisted")).toBe("function");
	function require() { return "function"; }
});

it("keeps nested block function compatibility names", () => {
	if (true) {
		{
			function require() { return "block"; }
		}
	}
	expect(require("./missing-block")).toBe("block");
});

it("keeps function names from switch, loop and catch scopes", () => {
	function fromSwitch() {
		switch (1) { case 1: function require() { return "switch"; } }
		return require("./missing-switch");
	}
	function fromLoop() {
		for (let i = 0; i < 1; i++) {
			function require() { return "loop"; }
		}
		return require("./missing-loop");
	}
	function fromCatch() {
		try { throw 1; } catch (_) {
			function require() { return "catch"; }
		}
		return require("./missing-catch");
	}
	expect(fromSwitch()).toBe("switch");
	expect(fromLoop()).toBe("loop");
	expect(fromCatch()).toBe("catch");
});

it("keeps body function names out of parameter defaults", () => {
	function separate(value = require("./value")) {
		if (true) { function require() { return "body"; } }
		return [value, require("./missing-body")];
	}
	expect(separate()).toEqual(["dependency", "body"]);
});

it("does not expose named expressions or class static block functions", () => {
	const named = function require() { return "expression"; };
	class Container {
		static value;
		static {
			{ function require() { return "static local"; } }
			this.value = require("./value");
		}
	}
	expect(named()).toBe("expression");
	expect(Container.value).toBe("dependency");
	expect(require("./value")).toBe("dependency");
});

it("preserves declaration hooks for function runtime names", () => {
	expect(__webpack_require__()).toBe("local runtime");
	function __webpack_require__() { return "local runtime"; }
	expect(require("./value")).toBe("dependency");
});

it("registers replacement AST functions from their own semantic information", () => {
	expect(REPLACEMENT).toBe("replacement");
	expect(REPLACEMENT_DEFAULT).toBe("replacement default");
	expect(require("./value")).toBe("dependency");
});
