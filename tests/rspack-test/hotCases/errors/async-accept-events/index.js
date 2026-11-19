require("./a");
require("./b");
require("./c");
require("./d");

module.hot.accept("./c", () => {
	throw new Error("c failed");
});
module.hot.accept("./d", () => {
	throw new Error("d failed");
});

it("should report and ignore synchronous errors with their original context", async () => {
	const events = [];
	await NEXT_HMR({
		ignoreErrored: true,
		onErrored: event => events.push(event)
	});
	expect(
		events.sort((a, b) =>
			a.moduleId.localeCompare(b.moduleId) ||
			a.dependencyId.localeCompare(b.dependencyId)
		)
	).toEqual([
		{
			type: "accept-errored",
			moduleId: "./a.js",
			dependencyId: "./module.js",
			error: expect.objectContaining({ message: "a failed" })
		},
		{
			type: "accept-error-handler-errored",
			moduleId: "./b.js",
			dependencyId: "./module.js",
			error: expect.objectContaining({ message: "b handler failed" }),
			originalError: expect.objectContaining({ message: "b failed" })
		},
		{
			type: "accept-errored",
			moduleId: "./index.js",
			dependencyId: "./c.js",
			error: expect.objectContaining({ message: "c failed" })
		},
		{
			type: "accept-errored",
			moduleId: "./index.js",
			dependencyId: "./d.js",
			error: expect.objectContaining({ message: "d failed" })
		}
	]);
	expect(module.hot.status()).toBe("idle");
});
