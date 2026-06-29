it("should work with cjs tree shaking and side effects free", () => {
	const lib = require("lib");
	const {} = require("lib");
	let b;
	if (FALSY) {
		b = lib.b;
	}
	expect(b).toBeUndefined();
});
