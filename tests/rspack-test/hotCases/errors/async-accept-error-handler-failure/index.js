require("./module");

it("should report an error handler throw after a synchronous accept throws", async () => {
	const originalError = new Error("accept failed");
	const handlerError = new Error("handler failed");
	module.hot.accept(
		"./module",
		() => {
			throw originalError;
		},
		() => {
			throw handlerError;
		}
	);
	const events = [];
	let failure;
	try {
		await NEXT_HMR({ onErrored: event => events.push(event) });
	} catch (err) {
		failure = err;
	}
	expect(failure).toBe(handlerError);
	expect(events).toEqual([
		{
			type: "accept-error-handler-errored",
			moduleId: "./index.js",
			dependencyId: "./module.js",
			error: handlerError,
			originalError
		}
	]);
	expect(module.hot.status()).toBe("fail");
});
