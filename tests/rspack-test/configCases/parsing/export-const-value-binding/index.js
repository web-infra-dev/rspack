import * as constExports from "./const-export";
import * as cyclicExports from "./cycle-a";
import * as functionExports from "./function-export";
import * as letExports from "./let-export";

it("should bind const exports as readonly values", () => {
	expectValueDescriptor(constExports, "literal", "literal");
	expectValueDescriptor(constExports, "renamed", "local");
	expectValueDescriptor(constExports, "destructured", "destructured");
	expectValueDescriptor(constExports, "arrayValue", "array");
});

it("should keep non-const exports as getters", () => {
	expectGetterDescriptor(letExports, "counter");
	expectGetterDescriptor(functionExports, "fn");
});

it("should keep const exports in circular modules as getters", () => {
	expectGetterDescriptor(cyclicExports, "cyclicConst");
	expect(cyclicExports.readFromCycle()).toBe("cyclic");
});

function expectValueDescriptor(ns, key, value) {
	const descriptor = Object.getOwnPropertyDescriptor(ns, key);
	expect(descriptor).toEqual(
		expect.objectContaining({
			enumerable: true,
			writable: false,
			value
		})
	);
	expect(descriptor.get).toBe(undefined);
}

function expectGetterDescriptor(ns, key) {
	const descriptor = Object.getOwnPropertyDescriptor(ns, key);
	expect(descriptor).toEqual(
		expect.objectContaining({
			enumerable: true,
			get: expect.any(Function)
		})
	);
	expect(Object.prototype.hasOwnProperty.call(descriptor, "value")).toBe(false);
}
