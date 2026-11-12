const events = require("./state");
require("./self");
require("./module");

it("should execute self-accepted modules after an accept rejection is handled", async () => {
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
	expect(events).toEqual(["initial", "self"]);
	expect(rejected).toBe(false);
	expect(failure).toBeUndefined();
	expect(handled).toBe(true);
	expect(reported).toEqual([]);
	expect(module.hot.status()).toBe("idle");
});
