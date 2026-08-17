"use strict";

let inheritedSetterValue;
const objectPrototype = Reflect.getPrototypeOf({});
const originalToLocaleString = Object.getOwnPropertyDescriptor(
	objectPrototype,
	"toLocaleString"
);
Object.defineProperty(objectPrototype, "toLocaleString", {
	configurable: true,
	set(value) {
		inheritedSetterValue = value;
	}
});
exports.toLocaleString = 42;
exports.value = inheritedSetterValue;
Object.defineProperty(objectPrototype, "toLocaleString", originalToLocaleString);
