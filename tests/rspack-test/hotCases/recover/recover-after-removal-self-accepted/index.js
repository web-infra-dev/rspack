import ok from "./module";

it("should abort when module is not accepted", async () => {
	expect(ok).toBe("ok1-inner");
	await NEXT_HMR();
	expect(ok).toBe("ok2");
	await NEXT_HMR();
	expect(ok).toBe("ok3-inner");
});

module.hot.accept("./module");
