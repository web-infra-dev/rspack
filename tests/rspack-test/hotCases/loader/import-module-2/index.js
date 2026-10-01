it("module and its loader-referencing module should update in right order", async () => {
	expect(require("./loader.mjs!./a")).toBe(2);
	await NEXT_HMR();
	expect(require("./loader.mjs!./a")).toBe(3);
});
module.hot.accept("./loader.mjs!./a");
