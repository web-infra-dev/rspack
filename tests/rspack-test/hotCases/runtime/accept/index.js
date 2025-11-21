var value = require("./file");

it("should accept a dependencies and require a new value", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	value = require("./file");
	expect(value).toBe(2);
	outside();
});

function outside() {
	expect(value).toBe(2);
}

module.hot.accept("./file");