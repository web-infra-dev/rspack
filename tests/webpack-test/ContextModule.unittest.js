"use strict";

// TODO: recover after we have this module
// const ContextModule = require("../lib/ContextModule");

describe.skip("contextModule", () => {
	let contextModule;
	let request;
	beforeEach(() => {
		request = "/some/request";
	});
	describe("#identifier", () => {
		it("returns an safe identifier for this module", () => {
			contextModule = new ContextModule(() => {}, {
				type: "javascript/auto",
				request,
				resource: "a",
				mode: "lazy",
				regExp: /a|b/
			});
			expect(contextModule.identifier()).toEqual(
				expect.stringContaining("/a%7Cb/")
			);
		});
	});
});
