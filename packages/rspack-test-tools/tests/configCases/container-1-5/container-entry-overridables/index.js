it("should expose modules from the container", async () => {
	const container = __non_webpack_require__("./container-file.js");
	expect(typeof container).toBe("object");
	expect(typeof container.init).toBe("function");
	container.init({
		value: {
			0: {
				get: () =>
					new Promise(resolve => {
						setTimeout(() => {
							resolve(() => ({
								__esModule: true,
								default: "overridden-value"
							}));
						}, 100);
					})
			}
		}
	});
	const testFactory = await container.get("./test");
	expect(typeof testFactory).toBe("function");
	expect(testFactory()).toEqual(
		nsObj({
			default: "test overridden-value"
		})
	);
});
