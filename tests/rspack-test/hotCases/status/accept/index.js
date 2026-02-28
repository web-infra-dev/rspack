var value = require("./file");

it("should wait until promises returned by status handlers are fulfilled", async () => {
	var handler = rstest.fn(status => {
		var test = rstest.fn(() => {
			expect(module.hot.status()).toBe(status == "dispose" ? "apply" : status);
		});

		var promise = Promise.resolve().then(test);
		promise.test = test;

		return promise;
	});
	module.hot.addStatusHandler(handler);
	await NEXT_HMR();
	// constructor not strict equal
	expect(handler.mock.calls).toEqual([['check'], ['prepare'], ['dispose'], ['apply'], ['idle']]);
	for (let result of handler.mock.results)
		expect(result.value.test).toHaveBeenCalledTimes(1);
	expect(module.hot.status()).toBe("idle");
});

module.hot.accept("./file", () => {
	value = require("./file");
});
