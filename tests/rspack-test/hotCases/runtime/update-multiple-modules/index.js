var value = require("./parent-file");

it("should update multiple modules at the same time", async () => {
	expect(value).toBe(2);
	await NEXT_HMR();
	value = require("./parent-file");
	expect(value).toBe(4);
});

module.hot.accept("./parent-file");