// change import order should rebuild chunk graph
import value from './value'
import lib from "./lib"
import { v1 } from './re-exports'

it("should have correct result", () => {
  expect(lib).toBe(42);
  expect(value).toBe(42);
	expect(v1).toBe(42);
});
