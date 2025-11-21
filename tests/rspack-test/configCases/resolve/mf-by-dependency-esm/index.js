import { a } from 'pkg';

it("should resolve to correct module", () => {
  expect(a).toBe(1);
})