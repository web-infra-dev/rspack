import { a } from './lib';
it("should run without `__webpack_require__(null)` error", () => {
  expect(a.value).toBe(42);
});
