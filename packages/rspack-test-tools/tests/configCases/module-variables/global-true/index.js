it("global true", function () {
	global;
	expect(__webpack_require__.g).not.toBe(undefined);
});

it("should polyfill `global` if `node.global` is `true`", function() {
	class Example {
		constructor(g = global) {
			this.g = g;
		}
	}
	expect(new Example().g).toBe(globalThis);
})


it("should only polyfill `global` if it's not declared", function() {
	class Example {
		constructor(global = global) {
			this.global = global;
		}
	}
	expect(() => new Example()).toThrow(/Cannot access '\w*' before initialization/);
})
