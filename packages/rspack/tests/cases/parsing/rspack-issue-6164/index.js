class Example {
  constructor(global = false) {
    this.global = global;
  }
}

it("should keep `global` as a local variable", function () {
	expect(new Example().global).toBe(false);
});
