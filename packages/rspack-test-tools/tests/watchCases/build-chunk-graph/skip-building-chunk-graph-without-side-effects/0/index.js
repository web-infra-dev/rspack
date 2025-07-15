// make sure
import lib from "./lib"
import { v1 } from './re-exports'
import value from './value'

it("should have correct result", () => {
  expect(value).toBe(42);
  expect(lib).toBe(42);
	expect(v1).toBe(42);
});
