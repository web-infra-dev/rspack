import { Kind } from './enum';
import { InlineableKind } from "./inlineable-enum";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should have dynamically defined methods (other exports should be null)", () => {
  expect(Kind.isA(Kind.A)).toBe(true);
  expect(Kind.isB(Kind.B)).toBe(true);
})

it("should inline enum if there is no usage of non-statical defined (no usage of other exports)", () => {
  // START:A
  expect(InlineableKind.A).toBe(0);
  expect(InlineableKind.B).toBe(1);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`(/* inlined export .InlineableKind.A */ (0)).toBe(0)`)).toBe(true);
  expect(block.includes(`(/* inlined export .InlineableKind.B */ (1)).toBe(1)`)).toBe(true);
  const inlinedModuleId = "./inlineable-enum.ts";
  expect(generated.includes(`"${inlinedModuleId}": (function`)).toBe(false);
})
