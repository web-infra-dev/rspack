"use strict";

let inheritedSetterValue;
const objectPrototype = Object.getPrototypeOf({});
const originalValueOf = Object.getOwnPropertyDescriptor(
	objectPrototype,
	"valueOf"
);
Object.defineProperty(objectPrototype, "valueOf", {
	configurable: true,
	set(value) {
		inheritedSetterValue = value;
	}
});
exports.valueOf = 42;
exports.value = inheritedSetterValue;
Object.defineProperty(objectPrototype, "valueOf", originalValueOf);
