it("should invalidate code generation when lazy and missing dependencies swap positions", async () => {
	const missing = COMPILER_INDEX === 1 ? "./a" : "./b";
	expect(() => require("./package")).toThrow(`Cannot find module '${missing}'`);
	if (COMPILER_INDEX < 3) await NEXT_START();
});
