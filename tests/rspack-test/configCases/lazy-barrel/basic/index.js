import { b as a, cc } from "./named-barrel";
import { b as c, d } from "./mixed-barrel";
import { b } from "./star-barrel";
import * as nested from "./nested-barrel";

it("should correct build", () => {
  expect(a).toBe('a');
  expect(cc).toBe('c');
  expect(b).toBe('b');
  expect(c).toBe('c');
  expect(d).toBe('d');
  expect(nested.a).toBe('b');
})
