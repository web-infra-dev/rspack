import value from "./callback";
import dependency from "./dependency";

it("should propagate asynchronous ESM errors without calling synchronous error handlers", async () => {
	expect([value, dependency]).toEqual([1, 1]);
	const errors = [];
	const callbackError = new Error("callback failed");
	import.meta.webpackHot.accept(
		"./callback",
		() => {
			throw callbackError;
		},
		(error, context) => {
			errors.push({ error, ...context });
		}
	);
	import.meta.webpackHot.accept(
		"./dependency",
		() => {
			throw new Error("should not call accept after a dependency rejects");
		},
		(error, context) => {
			errors.push({ error, ...context });
		}
	);
	const events = [];
	let failure;
	try {
		await NEXT_HMR({ onErrored: event => events.push(event) });
	} catch (err) {
		failure = err;
	}
	expect(failure).toBe(callbackError);
	expect(errors).toEqual([]);
	expect(value).toBe(2);
	expect(events).toEqual([]);
	expect(import.meta.webpackHot.status()).toBe("apply");
});
