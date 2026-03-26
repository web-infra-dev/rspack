import "./setup";
import "./consumer";

it("should keep destructuring-reassigned functions impure", () => {
  expect(globalThis.sideEffectCount).toBe(2);
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(5);
});
