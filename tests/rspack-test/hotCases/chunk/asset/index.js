var value = require("./file");
it("should accept a dependencies multiple times", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	while (true) {
		var oldValue = value;
		value = require("./file");
		expect(value).toBe(oldValue + 1);
		if (value < 4) {
			await NEXT_HMR();
		} else {
			break;
		}
	}
});

module.hot.accept("./file");