import lib from "./lib"
import value from './value'
import { v1 } from './re-exports'

it("should have correct result", () => {
	// add specifier dependency should not rebuild chunk graph
  expect(lib).toBe(42);
  expect(lib).toBe(42);
  expect(value).toBe(42);
	expect(v1).toBe(42);
});
