import * as enums from "./enum";
import * as reexported from "./re-export";
import * as destructuring from "./enum.destructuring";
import * as sideEffects from "./enum.side-effects";
import * as reexportedSideEffects from "./re-export.side-effects";
import * as notOnlyPropertiesUsed from "./enum.not-only-properties-used";
import * as exportAs from "./enum.export-as";
import * as reexportDestructuring from "./re-export.reexprt-destructuring";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should inline enums", () => {
  // START:A
  expect(enums.E.A).toBe(0);
  expect(enums.E.B).toBe(1);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`(/* inlined export .E.A */ (0)).toBe(0)`)).toBe(true);
  expect(block.includes(`(/* inlined export .E.B */ (1)).toBe(1)`)).toBe(true);
})

it("should inline enums with re-export", () => {
  // START:B
  expect(reexported.E.A).toBe(0);
  expect(reexported.E.B).toBe(1);
  // END:B
  const block = generated.match(/\/\/ START:B([\s\S]*)\/\/ END:B/)[1];
  expect(block.includes(`(/* inlined export .E.A */ (0)).toBe(0)`)).toBe(true);
  expect(block.includes(`(/* inlined export .E.B */ (1)).toBe(1)`)).toBe(true);
})

it("should not inline enums with destructuring", () => {
  // START:C
  const { A, B } = destructuring.E;
  expect(A).toBe(0);
  expect(B).toBe(1);
  // END:C
  const block = generated.match(/\/\/ START:C([\s\S]*)\/\/ END:C/)[1];
  expect(block.includes(`(A).toBe(0)`)).toBe(true);
  expect(block.includes(`(B).toBe(1)`)).toBe(true);
  expect(block.includes("inlined export")).toBe(false);
})

it("should allow inline enums if the rest exports is not used with destructuring", () => {
  // START:D
  expect(destructuring.E.C).toBe(2);
  expect(destructuring.E.D).toBe(3);
  // END:D
  const block = generated.match(/\/\/ START:D([\s\S]*)\/\/ END:D/)[1];
  expect(block.includes(`(/* inlined export .E.C */ (2)).toBe(2)`)).toBe(true);
  expect(block.includes(`(/* inlined export .E.D */ (3)).toBe(3)`)).toBe(true);
})

it("should respect side effects when inline enums", () => {
  // START:E
  expect(sideEffects.E.A).toBe(0);
  expect(globalThis.__sideEffects).toBe("enum.side-effects.ts");
  // END:E
  const block = generated.match(/\/\/ START:E([\s\S]*)\/\/ END:E/)[1];
  expect(block.includes(`(/* inlined export .E.A */ (0)).toBe(0)`)).toBe(true);
})

it("should respect side effects when inline enums with re-exports", () => {
  // START:F
  expect(reexportedSideEffects.E.A).toBe(0);
  // END:F
  const block = generated.match(/\/\/ START:F([\s\S]*)\/\/ END:F/)[1];
  expect(block.includes(`inlined export`)).toBe(true);
})

it("should not inline if enum is not only properties used", () => {
  // START:G
  ((e) => {
    expect(e.A).toBe(0);
    expect(e.B).toBe(1);
  })(notOnlyPropertiesUsed.E);
  expect(notOnlyPropertiesUsed.E.A).toBe(0);
  expect(notOnlyPropertiesUsed.E.B).toBe(1);
  // END:G
  const block = generated.match(/\/\/ START:G([\s\S]*)\/\/ END:G/)[1];
  expect(block.includes(`(e.A).toBe(0)`)).toBe(true);
  expect(block.includes(`(e.B).toBe(1)`)).toBe(true);
  expect(block.includes(`E.A).toBe(0)`)).toBe(true);
  expect(block.includes(`E.B).toBe(1)`)).toBe(true);
  expect(block.includes(`inlined export`)).toBe(false);
})

it("should remove the module if all enum members are inlined and side effects free", () => {
  const inlinedSideEffectsFreeModuleIds = ["./enum.ts", "./re-export.ts"];
  if (CONCATENATED) {
    inlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`;// CONCATENATED MODULE: ${m}`)).toBe(false);
    })
  } else {
    inlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`"${m}": (function`)).toBe(false);
    })
  }
})

it("should keep the module if all enum members are inlined but have side effects", () => {
  const inlinedSideEffectsNotFreeModuleIds = ["./enum.side-effects.ts", "./re-export.side-effects.ts"];
  if (CONCATENATED) {
    inlinedSideEffectsNotFreeModuleIds.forEach(m => {
      expect(generated.includes(`;// CONCATENATED MODULE: ${m}`)).toBe(true);
    })
  } else {
    inlinedSideEffectsNotFreeModuleIds.forEach(m => {
      expect(generated.includes(`"${m}": (function`)).toBe(true);
    })
  }
})

it("should keep the module if part of the enum members are inlined and side effects free", () => {
  const partialInlinedSideEffectsFreeModuleIds = ["./enum.destructuring.ts"];
  if (CONCATENATED) {
    partialInlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`;// CONCATENATED MODULE: ${m}`)).toBe(true);
    })
  } else {
    partialInlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`"${m}": (function`)).toBe(true);
    })
  }
})

it("should not inline no-inlinable enums", () => {
  expect(exportAs.InlineE1.A).toEqual({});
  expect(exportAs.InlineE.A).toBe(0);
})

it("should not inline for reexported destructuring", () => {
  expect(reexportDestructuring.e.E.A).toBe(reexportDestructuring.getValue());
})
