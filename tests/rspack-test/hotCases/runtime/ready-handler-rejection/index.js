import value from "./file";

it("should reject the check when a ready handler fails", async () => {
	const error = new Error("ready failed");
	module.hot.accept("./file");
	module.hot.addStatusHandler((status) => {
		if (status === "ready") throw error;
	});
	await expect(NEXT_HMR()).rejects.toBe(error);
	expect(module.hot.status()).toBe("abort");
	expect(value).toBe(1);
});
