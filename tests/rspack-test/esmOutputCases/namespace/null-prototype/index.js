import * as namespace from "./namespace.js";

it("should create namespace objects with a null prototype", () => {
  expect(Object.getPrototypeOf(namespace)).toBe(null);
  expect(Object.keys(namespace)).toEqual(["bar", "foo"]);
});

export const a = 1;
export const b = 2;
