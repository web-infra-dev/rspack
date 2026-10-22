it("should load consecutive updates from the output root", async () => {
	expect(require("./value")).toBe(1);

	await NEXT_HMR();
	expect(require("./value")).toBe(2);

	await NEXT_HMR();
	expect(require("./value")).toBe(3);
});

module.hot.accept("./value");
