import value from './value'
import lib from "./lib"
// import member from different modules should rebuild chunk graph
import { v1, v2, direct } from './re-exports'

it("should have correct result", () => {
  expect(lib).toBe(42);
  expect(value).toBe(42);
  expect(direct).toBe(42);
	expect(v1).toBe(42);
	expect(v2).toBe(42);
});
