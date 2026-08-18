import {
	getValue,
	anonymousFunction,
	anonymousArrow,
	AnonymousClass,
	defined,
	definedAnonymous,
	local,
	placeholder,
	readGlobal,
	readDefined,
	setValue,
	value
} from "./foo.js";

it("should scope-hoist a statically analyzable CommonJS module", () => {
	expect(value).toBe(1);
	expect(getValue()).toBe(1);
	expect(local).toBe(41);
	expect(placeholder).toBe(42);
	expect(readGlobal()).toBe(43);
	expect(anonymousFunction.name).toBe("");
	expect(anonymousArrow.name).toBe("");
	expect(AnonymousClass.name).toBe("");
	expect(defined).toBe(1);
	expect(definedAnonymous.name).toBe("");
	expect(readDefined()).toBe(44);

  setValue(2);

  expect(value).toBe(2);
  expect(getValue()).toBe(2);
});
