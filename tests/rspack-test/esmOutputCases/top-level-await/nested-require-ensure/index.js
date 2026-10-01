import { value } from "./lib.js";

// Keep a CommonJS module in the runtime chunk used by require.ensure.
const runtimeValue = require("./runtime.cjs");

const asyncModule = await new Promise((resolve, reject) => {
	require.ensure([], () => {
		require.ensure([], () => {
			import("./async.js").then(resolve, reject);
		}, error => reject(error));
	}, error => reject(error));
});

it("should extract shared modules for an awaited nested import", () => {
	expect(Object.keys(asyncModule)).toEqual(["value"]);
	expect(value).toBe(asyncModule.value);
	expect(value()).toBe(runtimeValue);
});
