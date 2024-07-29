import { a } from './lib';
debugger;
it("should run without `__webpack_require__(null)` error", () => {
  expect(a.value).toBe(42);
});
