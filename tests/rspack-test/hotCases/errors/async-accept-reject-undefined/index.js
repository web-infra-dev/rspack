require("./module");

it("should fail with an Error for a falsy accept rejection like webpack", async () => {
	const reason = undefined;
	module.hot.accept("./module", () => Promise.reject());
	let rejected = false;
	let failure;
	try {
		await NEXT_HMR();
	} catch (err) {
		rejected = true;
		failure = err;
	}
	expect(rejected).toBe(true);
	expect(failure).toEqual(new Error("Update failed with " + reason));
	expect(module.hot.status()).toBe("fail");
});
