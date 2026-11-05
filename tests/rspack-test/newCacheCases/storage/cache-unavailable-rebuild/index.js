import value from "./a";

it("should keep rebuilding with the file cache disabled after a commit failure", async () => {
	if (COMPILER_INDEX === 0) {
		expect(value).toBe("a");
		await cacheUnavailable;
		await NEXT_HMR();
		expect(value).toBe("b");
		await NEXT_HMR();
		expect(value).toBe("c");
		await NEXT_START();
	} else {
		expect(value).toBe("c");
	}
});

module.hot.accept("./a");
