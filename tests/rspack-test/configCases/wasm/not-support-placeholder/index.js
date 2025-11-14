import { _Z4facti } from './factorial.wasm';

const factorial = _Z4facti;

it("should compile wasm", () => {
  expect(factorial(3)).toBe(6);
});
