require("./module");

it("should remain in apply status when an accept promise rejects", async () => {
	const error = new Error("async accept failed");
	module.hot.accept("./module", async () => {
		throw error;
	});
	let failure;
	try {
		await NEXT_HMR();
	} catch (err) {
		failure = err;
	}
	expect(failure).toBe(error);
	expect(module.hot.status()).toBe("apply");
});
