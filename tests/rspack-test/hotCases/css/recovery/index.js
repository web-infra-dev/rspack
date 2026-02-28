import "./main";

it("css recovery", async () => {
	try {
		await NEXT_HMR();
	} catch (err) {
		expect(String(err)).toContain("Module build failed");
		await NEXT_HMR();
	}
});

module.hot.accept("./main");
