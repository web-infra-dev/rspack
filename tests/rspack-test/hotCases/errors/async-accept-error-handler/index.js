import value from "./callback";
import dependency from "./dependency";

it("should handle asynchronous ESM and callback errors with their dependency context", async () => {
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
	expect(failure).toBeUndefined();
	expect(errors).toEqual(
		expect.arrayContaining([
			{
				error: callbackError,
				moduleId: "./index.js",
				dependencyId: "./callback.js"
			},
			{
				error: expect.objectContaining({ message: "dependency failed" }),
				moduleId: "./index.js",
				dependencyId: "./dependency.js"
			}
		])
	);
	expect(errors).toHaveLength(2);
	expect(value).toBe(2);
	expect(events).toEqual([]);
	expect(import.meta.webpackHot.status()).toBe("idle");
});
