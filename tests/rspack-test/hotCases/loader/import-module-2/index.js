it("module and its loader-referencing module should update in right order", async () => {
	expect(require("./loader.js!./a")).toBe(2);
	await NEXT_HMR();
	expect(require("./loader.js!./a")).toBe(3);
});
module.hot.accept("./loader.js!./a");
