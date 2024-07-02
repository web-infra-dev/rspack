/**
 * The following code is modified based on
 * https://github.com/webpack/tapable/blob/a0a7b26/lib/__tests__/SyncHooks.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const HookTester = require("./HookTester");
const { SyncHook, SyncBailHook, SyncWaterfallHook } = require("../");

describe("SyncHook", () => {
	it("should have to correct behavior", async () => {
		const tester = new HookTester(args => new SyncHook(args));

		const result = await tester.run(true);

		expect(result).toMatchSnapshot();
	}, 15000);
});

describe("SyncBailHook", () => {
	it("should have to correct behavior", async () => {
		const tester = new HookTester(args => new SyncBailHook(args));

		const result = await tester.run(true);

		expect(result).toMatchSnapshot();
	}, 15000);
});

describe("SyncWaterfallHook", () => {
	it("should have to correct behavior", async () => {
		const tester = new HookTester(args => new SyncWaterfallHook(args));

		const result = await tester.run(true);

		expect(result).toMatchSnapshot();
	}, 15000);
});

// describe("SyncLoopHook", () => {
// 	it("should have to correct behavior", async () => {
// 		const tester = new HookTester(args => new SyncLoopHook(args));

// 		const result = await tester.runForLoop(true);

// 		expect(result).toMatchSnapshot();
// 	}, 15000);
// });
