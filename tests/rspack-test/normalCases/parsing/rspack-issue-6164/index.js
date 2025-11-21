it("should keep `global` as a local variable", function () {
	class Example {
		constructor(global = false) {
			this.global = global;
		}
	}
	expect(new Example().global).toBe(false);
});
