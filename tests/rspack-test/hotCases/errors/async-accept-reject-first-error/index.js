require("./a");
require("./b");

it("should preserve the first rejection when a later accept rejects with an Error", async () => {
	const laterError = new Error("later accept failed");
	module.hot.accept("./a", () => Promise.reject());
	module.hot.accept("./b", async () => {
		await Promise.resolve();
		throw laterError;
	});
	const errors = [];
	let rejected = false;
	let failure;
	try {
		await NEXT_HMR({ onErrored: event => errors.push(event.error) });
	} catch (err) {
		rejected = true;
		failure = err;
	}
	expect(errors).toEqual([]);
	expect(rejected).toBe(true);
	expect(failure).toBeUndefined();
	expect(module.hot.status()).toBe("apply");
});
