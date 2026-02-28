import value from "./file";

it("should fix issue 10915", async () => {
	expect(value).toBe("file-abcd");
	await NEXT_HMR();
	expect(value).toBe("file");
});

module.hot.accept("./file");