import {
	getValue,
	local,
	placeholder,
	readGlobal,
	setValue,
	value
} from "./foo.js";

it("should scope-hoist a statically analyzable CommonJS module", () => {
	expect(value).toBe(1);
	expect(getValue()).toBe(1);
	expect(local).toBe(41);
	expect(placeholder).toBe(42);
	expect(readGlobal()).toBe(43);

  setValue(2);

  expect(value).toBe(2);
  expect(getValue()).toBe(2);
});
