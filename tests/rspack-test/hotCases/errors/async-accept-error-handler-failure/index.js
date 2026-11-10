require("./module");

it("should report an error handler throw after an async accept rejects", async () => {
	const originalError = new Error("accept failed");
	const handlerError = new Error("handler failed");
	module.hot.accept(
		"./module",
		async () => {
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
