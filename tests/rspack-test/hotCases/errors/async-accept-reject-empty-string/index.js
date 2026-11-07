require("./module");

it("should preserve a falsy accept rejection and remain in apply status", async () => {
	const reason = "";
	module.hot.accept("./module", () => Promise.reject(reason));
	let rejected = false;
	let failure;
	try {
		await NEXT_HMR();
	} catch (err) {
		rejected = true;
		failure = err;
	}
	expect(rejected).toBe(true);
	expect(failure).toBe(reason);
	expect(module.hot.status()).toBe("apply");
});
