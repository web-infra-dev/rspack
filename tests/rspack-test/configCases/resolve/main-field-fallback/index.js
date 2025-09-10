import { foo } from 'invalid-pkg'

it("should fallback from module to main when resolve failed", () => {
  expect(foo).toBe(42);
});