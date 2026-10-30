require("./module");

it("should reject the update when an accept promise rejects", async () => {
	const error = new Error("async accept failed");
	const rejected = Promise.reject(error);
	rejected.catch(() => {});
	module.hot.accept("./module", () => rejected);
	let failure;
	try {
		await NEXT_HMR();
	} catch (err) {
		failure = err;
	}
	expect(failure).toBe(error);
});
