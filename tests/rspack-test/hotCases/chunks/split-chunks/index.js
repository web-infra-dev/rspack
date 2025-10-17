import vendor from "vendor";
it("should hot update a splitted initial chunk", async () => {
	expect(vendor).toBe("1");
	await NEXT_HMR();
	expect(vendor).toBe("2");
});

module.hot.accept(["vendor"]);