import { E1, E2 } from "./enum";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should not inline for non-const enum for const-only mode", () => {
  // START:A
  expect(E1.A).toBe(0);
  expect(E1.B).toBe(1);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`E1.A).toBe(0)`)).toBe(true);
  expect(block.includes(`E1.B).toBe(1)`)).toBe(true);
})

it("should inline for const enum for const-only mode", () => {
  // START:B
  expect(E2.A).toBe(0);
  expect(E2.B).toBe(1);
  // END:B
  const block = generated.match(/\/\/ START:B([\s\S]*)\/\/ END:B/)[1];
  expect(block.includes(`(/* inlined export .E2.A */ (0)).toBe(0)`)).toBe(true);
  expect(block.includes(`(/* inlined export .E2.B */ (1)).toBe(1)`)).toBe(true);
})
