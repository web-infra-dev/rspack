"use strict";

// TODO: recover after we have this module
// const AbstractMethodError = require("../lib/AbstractMethodError");

describe.skip("WebpackError", () => {
	class Foo {
		abstractMethod() {
			return new AbstractMethodError();
		}
	}

	class Child extends Foo {}

	const expectedMessage = "Abstract method $1. Must be overridden.";

	it("Should construct message with caller info", () => {
		const fooClassError = new Foo().abstractMethod();
		const childClassError = new Child().abstractMethod();

		expect(fooClassError.message).toBe(
			expectedMessage.replace("$1", "Foo.abstractMethod")
		);
		expect(childClassError.message).toBe(
			expectedMessage.replace("$1", "Child.abstractMethod")
		);
	});
});
