require("./module");

it("should propagate onErrored throws for a synchronous accept error", async () => {
	const acceptError = new Error("accept failed");
	const reportError = new Error("onErrored failed");
	module.hot.accept("./module", () => {
		throw acceptError;
	});
	let failure;
	try {
		await NEXT_HMR({
			onErrored(event) {
				expect(event.error).toBe(acceptError);
				throw reportError;
			}
		});
	} catch (err) {
		failure = err;
	}
	expect(failure).toBe(reportError);
	expect(module.hot.status()).toBe("apply");
});
