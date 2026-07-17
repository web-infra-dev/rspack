it("demotes a cached empty auto module after the static ESM edge disappears", () => {
	expect(require("./empty").__esModule).toBeUndefined();
});
