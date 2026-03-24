import * as ns from "./foo.js";

it("should expose namespace keys in spec order", () => {
  expect(Object.getOwnPropertyNames(ns)).toEqual([
    "$",
    "$$$",
    "A",
    "AA",
    "Z",
    "ZZZ",
    "___",
    "aa",
    "default",
    "foo",
    "namespace",
    "z",
    "ö",
    "ø",
    "ς",
  ]);
});
