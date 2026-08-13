"use strict";

let inheritedSetterValue;
const originalToString = Object.getOwnPropertyDescriptor(
	Object.prototype,
	"toString"
);
Object.defineProperty(Object.prototype, "toString", {
	configurable: true,
	set(value) {
		inheritedSetterValue = value;
	}
});
exports.toString = 42;
exports.value = inheritedSetterValue;
Object.defineProperty(Object.prototype, "toString", originalToString);
