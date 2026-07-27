let currentModule = require("./module");

it("resets hot data when a removed module is added again", async () => {
	const initialData = currentModule.default;
	expect(initialData.marker).toBe("initial");

	await NEXT_HMR();
	require("./module");
	expect(initialData.disposed).toBe(true);

	await NEXT_HMR();
	currentModule = require("./module");
	expect(currentModule.default).not.toBe(initialData);
	expect(currentModule.default).toEqual({});
});

module.hot.accept("./module");
