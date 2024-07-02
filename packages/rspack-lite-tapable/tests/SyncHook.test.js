/**
 * The following code is modified based on
 * https://github.com/webpack/tapable/blob/a0a7b26/lib/__tests__/SyncHook.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const { SyncHook } = require("../");

describe("SyncHook", () => {
	it("should allow to create sync hooks", async () => {
		const h0 = new SyncHook();
		const h1 = new SyncHook(["test"]);
		const h2 = new SyncHook(["test", "arg2"]);
		const h3 = new SyncHook(["test", "arg2", "arg3"]);

		h0.call();
		await h0.promise();
		await new Promise(resolve => h0.callAsync(resolve));

		const mock0 = jest.fn();
		h0.tap("A", mock0);

		h0.call();

		expect(mock0).toHaveBeenLastCalledWith();

		const mock1 = jest.fn();
		h0.tap("B", mock1);

		h0.call();

		expect(mock1).toHaveBeenLastCalledWith();

		const mock2 = jest.fn();
		const mock3 = jest.fn();
		const mock4 = jest.fn();
		const mock5 = jest.fn();

		h1.tap("C", mock2);
		h2.tap("D", mock3);
		h3.tap("E", mock4);
		h3.tap("F", mock5);

		h1.call("1");
		h2.call("1", 2);
		h3.call("1", 2, 3);

		expect(mock2).toHaveBeenLastCalledWith("1");
		expect(mock3).toHaveBeenLastCalledWith("1", 2);
		expect(mock4).toHaveBeenLastCalledWith("1", 2, 3);
		expect(mock5).toHaveBeenLastCalledWith("1", 2, 3);

		await new Promise(resolve => h1.callAsync("a", resolve));
		await h2.promise("a", "b");
		await new Promise(resolve => h3.callAsync("a", "b", "c", resolve));

		expect(mock2).toHaveBeenLastCalledWith("a");
		expect(mock3).toHaveBeenLastCalledWith("a", "b");
		expect(mock4).toHaveBeenLastCalledWith("a", "b", "c");
		expect(mock5).toHaveBeenLastCalledWith("a", "b", "c");

		await h3.promise("x", "y");

		expect(mock4).toHaveBeenLastCalledWith("x", "y", undefined);
		expect(mock5).toHaveBeenLastCalledWith("x", "y", undefined);
	});

	it("should sync execute hooks", () => {
		const h1 = new SyncHook(["a"]);
		const mockCall1 = jest.fn();
		const mockCall2 = jest.fn(() => "B");
		const mockCall3 = jest.fn(() => "C");
		h1.tap("A", mockCall1);
		h1.tap("B", mockCall2);
		h1.tap("C", mockCall3);
		expect(h1.call()).toEqual(undefined);
		expect(mockCall1).toHaveBeenCalledTimes(1);
		expect(mockCall2).toHaveBeenCalledTimes(1);
		expect(mockCall3).toHaveBeenCalledTimes(1);
	});

	it("should allow to intercept calls", () => {
		const hook = new SyncHook(["arg1", "arg2"]);

		const mockCall = jest.fn();
		const mock0 = jest.fn();
		const mockRegister = jest.fn(x => ({
			name: "huh",
			type: "sync",
			fn: mock0
		}));

		const mock1 = jest.fn();
		hook.tap("Test1", mock1);

		hook.intercept({
			call: mockCall,
			register: mockRegister
		});

		const mock2 = jest.fn();
		hook.tap("Test2", mock2);

		hook.call(1, 2);

		expect(mockCall).toHaveBeenLastCalledWith(1, 2);
		expect(mockRegister).toHaveBeenLastCalledWith({
			type: "sync",
			name: "Test2",
			fn: mock2
		});
		expect(mock1).not.toHaveBeenLastCalledWith(1, 2);
		expect(mock2).not.toHaveBeenLastCalledWith(1, 2);
		expect(mock0).toHaveBeenLastCalledWith(1, 2);
	});

	it("should throw error on tapAsync", () => {
		const hook = new SyncHook(["arg1", "arg2"]);
		expect(() => hook.tapAsync()).toThrow(/tapAsync/);
	});

	it("should throw error on tapPromise", () => {
		const hook = new SyncHook(["arg1", "arg2"]);
		expect(() => hook.tapPromise()).toThrow(/tapPromise/);
	});
});
