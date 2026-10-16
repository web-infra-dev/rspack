import value from "./file";

it("should reject the check when a prepare handler fails", async () => {
	const error = new Error("prepare failed");
	module.hot.accept("./file");
	module.hot.addStatusHandler((status) => {
		if (status === "prepare") return Promise.reject(error);
	});
	await expect(NEXT_HMR()).rejects.toBe(error);
	expect(module.hot.status()).toBe("abort");
	expect(value).toBe(1);
});
