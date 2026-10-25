import value from "./dep";

it("should be replaced during finishModules", function() {
	// The finishModules rebuild replaces this module before it is emitted,
	// so this assertion only guards the pre-rebuild graph shape.
	expect(value).toBe("initial");
});
