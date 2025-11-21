it("should expose modules from the container", async () => {
	const container = __non_webpack_require__("./container-file.js");
	expect(typeof container).toBe("object");
	expect(typeof container.get).toBe("function");
	const testFactory = await container.get("./test");
	expect(typeof testFactory).toBe("function");
	expect(testFactory()).toBe("test");
	const mainFactory = await container.get(".");
	expect(typeof mainFactory).toBe("function");
	expect(mainFactory()).toBe("main");
	const test2Factory = await container.get("./test2");
	expect(typeof test2Factory).toBe("function");
	expect(test2Factory()).toEqual(
		nsObj({
			default: "test2",
			other: "other"
		})
	);
});
