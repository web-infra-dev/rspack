it("should be able to recover from json error", async () => {
	expect(() => require("./data.json")).toThrowError();
	await NEXT_HMR();
	expect(require("./data.json")).toBe(42);
});

module.hot.accept("./data.json");