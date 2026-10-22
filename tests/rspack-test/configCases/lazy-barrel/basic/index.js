import { b as a, cc } from "./named-barrel";
import localDefault, { b as c, d } from "./mixed-barrel";
import { b } from "./star-barrel";
import * as nested from "./nested-barrel";

it("should correct build", () => {
  expect(a).toBe('a');
  expect(cc).toBe('c');
  expect(b).toBe('b');
  expect(c).toBe('c');
  expect(d).toBe('d');
  expect(localDefault()).toBe('default');
  expect(nested.a).toBe('b');
})
