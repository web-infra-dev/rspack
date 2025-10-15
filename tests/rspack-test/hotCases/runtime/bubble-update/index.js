var value = require("./parent-file");

it("should bubble update from a nested dependency", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	value = require("./parent-file");
	expect(value).toBe(2);
});

module.hot.accept("./parent-file");
