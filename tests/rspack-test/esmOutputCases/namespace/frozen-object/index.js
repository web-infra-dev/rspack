import * as ns from "./mod.js";

function extend(object) {
  "use strict";
  object.newProperty = true;
}

function reconfigure(object) {
  Object.defineProperty(object, "a", { value: null });
}

function mutate(object) {
  "use strict";
  object.a = 2;
}

it("should keep namespace objects non-extensible and immutable", () => {
  expect(Object.isExtensible(ns)).toBe(false);
  expect(() => extend(ns)).toThrow();
  expect(() => reconfigure(ns)).toThrow();
  expect(() => mutate(ns)).toThrow();
});

export { ns };
