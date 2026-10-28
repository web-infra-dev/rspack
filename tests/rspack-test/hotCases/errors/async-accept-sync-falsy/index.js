require("./module");

it("should ignore a synchronous falsy throw like webpack", async () => {
	module.hot.accept("./module", () => { throw 0; });
	const errors = [];
	let rejected = false;
	try {
		await NEXT_HMR({ onErrored: event => errors.push(event.error) });
	} catch (err) {
		rejected = true;
	}
	expect(rejected).toBe(false);
	expect(errors).toEqual([0]);
	expect(module.hot.status()).toBe("idle");
});
