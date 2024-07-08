/**
 * The following code is modified based on
 * https://github.com/webpack/tapable/blob/a0a7b26/lib/__tests__/AsyncParallelHooks.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const HookTester = require("./HookTester");
const { AsyncParallelHook } = require("../");

describe("AsyncParallelHook", () => {
	it("should have to correct behavior", async () => {
		const tester = new HookTester(args => new AsyncParallelHook(args));

		const result = await tester.run();

		expect(result).toMatchSnapshot();
	}, 15000);
});

// describe("AsyncParallelBailHook", () => {
// 	it("should have to correct behavior", async () => {
// 		const tester = new HookTester(args => new AsyncParallelBailHook(args));

// 		const result = await tester.run();

// 		expect(result).toMatchSnapshot();
// 	}, 15000);
// });
