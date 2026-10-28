it("should resolve custom update paths relative to the runtime chunk", async () => {
	expect(require("./value")).toBe(1);
	expect((await import("./lazy")).default).toBe("lazy");

	await NEXT_HMR();
	expect(require("./value")).toBe(2);

	await NEXT_HMR();
	expect(require("./value")).toBe(3);
});

module.hot.accept("./value");
