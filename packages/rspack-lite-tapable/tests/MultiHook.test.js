/**
 * The following code is modified based on
 * https://github.com/webpack/tapable/blob/a0a7b26/lib/__tests__/MultiHook.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const { MultiHook } = require("../");

describe("MultiHook", () => {
	const redirectedMethods = ["tap", "tapAsync", "tapPromise"];
	for (const name of redirectedMethods) {
		it(`should redirect ${name}`, () => {
			const calls = [];
			const fakeHook = {
				[name]: (options, fn) => {
					calls.push({ options, fn });
				}
			};
			new MultiHook([fakeHook, fakeHook])[name]("options", "fn");
			expect(calls).toEqual([
				{ options: "options", fn: "fn" },
				{ options: "options", fn: "fn" }
			]);
		});
	}
	it("should redirect intercept", () => {
		const calls = [];
		const fakeHook = {
			intercept: interceptor => {
				calls.push(interceptor);
			}
		};
		new MultiHook([fakeHook, fakeHook]).intercept("interceptor");
		expect(calls).toEqual(["interceptor", "interceptor"]);
	});
	// it("should redirect withOptions", () => {
	// 	const calls = [];
	// 	const fakeHook = {
	// 		withOptions: options => {
	// 			calls.push(options);
	// 			return {
	// 				tap: (options, fn) => {
	// 					calls.push({ options, fn });
	// 				}
	// 			};
	// 		}
	// 	};
	// 	const newHook = new MultiHook([fakeHook, fakeHook]).withOptions("options");
	// 	newHook.tap("options", "fn");
	// 	expect(calls).toEqual([
	// 		"options",
	// 		"options",
	// 		{ options: "options", fn: "fn" },
	// 		{ options: "options", fn: "fn" }
	// 	]);
	// });
	it("should redirect isUsed", () => {
		const calls = [];
		const fakeHook1 = {
			isUsed: () => {
				return true;
			}
		};
		const fakeHook2 = {
			isUsed: () => {
				return false;
			}
		};
		expect(new MultiHook([fakeHook1, fakeHook1]).isUsed()).toEqual(true);
		expect(new MultiHook([fakeHook1, fakeHook2]).isUsed()).toEqual(true);
		expect(new MultiHook([fakeHook2, fakeHook1]).isUsed()).toEqual(true);
		expect(new MultiHook([fakeHook2, fakeHook2]).isUsed()).toEqual(false);
	});
});
