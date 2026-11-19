require("./a");
require("./b");

it("should report all rejections and fail with the first error like webpack", async () => {
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
	expect(errors).toEqual([undefined, laterError]);
	expect(rejected).toBe(true);
	expect(failure).toEqual(new Error("Update failed with undefined"));
	expect(module.hot.status()).toBe("fail");
});
