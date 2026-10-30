const events = require("./state");
require("./self");
require("./module");

it("should skip self-accepted modules when an accept promise rejects", async () => {
	const error = new Error("accept failed");
	let handled = false;
	const reported = [];
	module.hot.accept(
		"./module",
		() => Promise.reject(error),
		() => { handled = true; }
	);
	let rejected = false;
	let failure;
	try {
		await NEXT_HMR({ ignoreErrored: true, onErrored: event => reported.push(event) });
	} catch (err) {
		rejected = true;
		failure = err;
	}
	expect(events).toEqual(["initial"]);
	expect(rejected).toBe(true);
	expect(failure).toBe(error);
	expect(handled).toBe(false);
	expect(reported).toEqual([]);
	expect(module.hot.status()).toBe("apply");
});
