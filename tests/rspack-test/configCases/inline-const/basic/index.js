import * as constants from "./constants.js";
import * as reexported from "./re-export.js";
import * as destructing from "./constants.destructing.js";
import * as sideEffects from "./constants.side-effects.js";
import * as reexportedSideEffects from "./re-export.side-effects.js";
import * as reexportedBarrelSideEffects from "./re-export.barrel-side-effects.js";
import * as reexportedDestructingBarrelSideEffects from "./re-export.destructing-barrel-side-effects.js";
import * as constantsCjs from "./constants.cjs";
import * as constantsNoInline from "./constants.no-inline.js";
import { REMOVE_b as BRANCH_TRUE, REMOVE_FALSE as BRANCH_FALSE } from "./constants.js";

const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should inline constants", () => {
  // START:A
  expect(constants.REMOVE_n).toBe(null);
  expect(constants.REMOVE_u).toBe(undefined);
  expect(constants.REMOVE_b).toBe(true);
  expect(constants.REMOVE_i).toBe(123456);
  expect(constants.REMOVE_f).toBe(123.45);
  expect(constants.REMOVE_s).toBe("remove");
  expect(constants.REMOVE_m).toBe(13);
  // END:A
  const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
  expect(block.includes(`((/* inlined export .REMOVE_n */null)).toBe(null)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_u */undefined)).toBe(undefined)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_b */true)).toBe(true)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_i */123456)).toBe(123456)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_f */123.45)).toBe(123.45)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_s */"remove")).toBe("remove")`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_m */13)).toBe(13)`)).toBe(true);
})

it("should inline constants with re-export", () => {
  // START:B
  expect(reexported.REMOVE_n).toBe(null);
  expect(reexported.REMOVE_u).toBe(undefined);
  expect(reexported.REMOVE_b).toBe(true);
  expect(reexported.REMOVE_i).toBe(123456);
  expect(reexported.REMOVE_f).toBe(123.45);
  expect(reexported.REMOVE_s).toBe("remove");
  // END:B
  const block = generated.match(/\/\/ START:B([\s\S]*)\/\/ END:B/)[1];
  expect(block.includes(`((/* inlined export .REMOVE_n */null)).toBe(null)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_u */undefined)).toBe(undefined)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_b */true)).toBe(true)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_i */123456)).toBe(123456)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_f */123.45)).toBe(123.45)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_s */"remove")).toBe("remove")`)).toBe(true);
})

it("should not inline constants with destructing", () => {
  // START:C
  const { REMOVE_n, REMOVE_u, REMOVE_b } = destructing;
  expect(REMOVE_n).toBe(null);
  expect(REMOVE_u).toBe(undefined);
  expect(REMOVE_b).toBe(true);
  // END:C
  const block = generated.match(/\/\/ START:C([\s\S]*)\/\/ END:C/)[1];
  expect(block.includes(`(REMOVE_n).toBe(null)`)).toBe(true);
  expect(block.includes(`(REMOVE_u).toBe(undefined)`)).toBe(true);
  expect(block.includes(`(REMOVE_b).toBe(true)`)).toBe(true);
  expect(block.includes("inlined export")).toBe(false);
})

it("should allow inline constants if the rest exports is not used with destructing", () => {
  // START:D
  expect(destructing.REMOVE_i).toBe(123456);
  expect(destructing.REMOVE_f).toBe(123.45);
  expect(destructing.REMOVE_s).toBe("remove");
  // END:D
  const block = generated.match(/\/\/ START:D([\s\S]*)\/\/ END:D/)[1];
  expect(block.includes(`((/* inlined export .REMOVE_i */123456)).toBe(123456)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_f */123.45)).toBe(123.45)`)).toBe(true);
  expect(block.includes(`((/* inlined export .REMOVE_s */"remove")).toBe("remove")`)).toBe(true);
})

it("should respect side effects when inline constants", () => {
  // START:E
  expect(sideEffects.REMOVE_CONST).toBe(true);
  expect(globalThis.__sideEffects).toBe("constants.side-effects.js");
  // END:E
  const block = generated.match(/\/\/ START:E([\s\S]*)\/\/ END:E/)[1];
  expect(block.includes(`((/* inlined export .REMOVE_CONST */true)).toBe(true)`)).toBe(true);
})

it("should not inline and link to re-export module when have side effects", () => {
  // START:F
  expect(reexportedSideEffects.REMOVE_CONST).toBe(true);
  // END:F
  const block = generated.match(/\/\/ START:F([\s\S]*)\/\/ END:F/)[1];
  if (CONCATENATED) {
    expect(block.includes(`inlined export`)).toBe(true);
  } else {
    expect(block.includes(`inlined export`)).toBe(false);
  }
})

it("should not inline and link to re-export module when have barrel side effects", () => {
  // START:G
  expect(reexportedBarrelSideEffects.REMOVE_s).toBe("remove");
  // END:G
  expect(globalThis.__barrelSideEffects).toBe("re-export.barrel-side-effects.js");
  const block = generated.match(/\/\/ START:G([\s\S]*)\/\/ END:G/)[1];
  if (CONCATENATED) {
    expect(block.includes(`inlined export`)).toBe(true);
  } else {
    const code = generated.match(/"\.\/re-export\.barrel-side-effects\.js"\(.*{([\s\S]*?)},/)[1];
    expect(code.includes(`__webpack_require__("./constants.js")`)).toBe(false);
    expect(block.includes(`inlined export`)).toBe(false);
  }
})

it("should not inline destructing with re-export", () => {
  // START:H
  const { REMOVE_n, REMOVE_u, REMOVE_b } = reexportedDestructingBarrelSideEffects.m;
  expect(REMOVE_n).toBe(null);
  expect(REMOVE_u).toBe(undefined);
  expect(REMOVE_b).toBe(true);
  expect(reexportedDestructingBarrelSideEffects.m.REMOVE_i).toBe(123456);
  expect(reexportedDestructingBarrelSideEffects.m.REMOVE_f).toBe(123.45);
  expect(reexportedDestructingBarrelSideEffects.m.REMOVE_s).toBe("remove");
  // END:H
  expect(globalThis.__destructingBarrelSideEffects).toBe("re-export.destructing-barrel-side-effects.js");
  const block = generated.match(/\/\/ START:H([\s\S]*)\/\/ END:H/)[1];
  expect(block.includes(`(REMOVE_n).toBe(null)`)).toBe(true);
  expect(block.includes(`(REMOVE_u).toBe(undefined)`)).toBe(true);
  expect(block.includes(`(REMOVE_b).toBe(true)`)).toBe(true);
  expect(block.includes(`.REMOVE_i */123456)).toBe(123456)`)).toBe(true);
  expect(block.includes(`.REMOVE_f */123.45)).toBe(123.45)`)).toBe(true);
  expect(block.includes(`.REMOVE_s */"remove")).toBe("remove")`)).toBe(true);
})

it("should not inline for cjs", () => {
  expect(constantsCjs.REMOVE_CONST).toBe(true);
  const cjsModuleIds = ["./constants.cjs"];
  cjsModuleIds.forEach(m => {
    expect(generated.includes(`"${m}"(`)).toBe(true);
  })
})

it("should remove the module if all exports is inlined and side effects free", () => {
  const inlinedSideEffectsFreeModuleIds = ["./constants.js", "./re-export.js"];
  if (CONCATENATED) {
    inlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`;// CONCATENATED MODULE: ${m}`)).toBe(false);
    })
  } else {
    inlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`"${m}"(`)).toBe(false);
    })
  }
})

it("should keep the module if all exports is inlined but have side effects", () => {
  const inlinedSideEffectsNotFreeModuleIds = ["./constants.side-effects.js", "./re-export.side-effects.js"];
  if (CONCATENATED) {
    inlinedSideEffectsNotFreeModuleIds.forEach(m => {
      expect(generated.includes(`;// CONCATENATED MODULE: ${m}`)).toBe(true);
    })
  } else {
    inlinedSideEffectsNotFreeModuleIds.forEach(m => {
      expect(generated.includes(`"${m}"(`)).toBe(true);
    })
  }
})

it("should keep the module if part of the exports is inlined and side effects free", () => {
  const partialInlinedSideEffectsFreeModuleIds = ["./constants.destructing.js"];
  if (CONCATENATED) {
    partialInlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`;// CONCATENATED MODULE: ${m}`)).toBe(true);
    })
  } else {
    partialInlinedSideEffectsFreeModuleIds.forEach(m => {
      expect(generated.includes(`"${m}"(`)).toBe(true);
    })
  }
})

it("should not inline no-inlinable constants", () => {
  expect(constantsNoInline.INLINE_1).toEqual({});
})

it("should drop branch dependencies guarded by inlined imported constants", () => {
  if (BRANCH_TRUE) {
    expect(BRANCH_TRUE).toBe(true);
  } else {
    require("./branch-unused.js");
  }
  if (!BRANCH_TRUE) {
    require("./branch-unused.js");
  } else {
    expect(BRANCH_TRUE).toBe(true);
  }
  if (BRANCH_FALSE) {
    require("./branch-unused.js");
  } else {
    expect(require("./branch-used.js")).toBe("used");
  }
  if (!BRANCH_FALSE) {
    expect(BRANCH_FALSE).toBe(false);
  } else {
    require("./branch-unused.js");
  }
  if (BRANCH_TRUE && BRANCH_FALSE) {
    require("./branch-unused.js");
  } else {
    expect(require("./branch-logical-used.js")).toBe("logical-used");
  }
  if (BRANCH_TRUE || BRANCH_FALSE) {
    expect(require("./branch-logical-used.js")).toBe("logical-used");
  } else {
    require("./branch-unused.js");
  }
  if (BRANCH_TRUE && false) {
    require("./branch-unused.js");
  } else {
    expect(require("./branch-logical-used.js")).toBe("logical-used");
  }
  if (false || BRANCH_FALSE) {
    require("./branch-unused.js");
  } else {
    expect(BRANCH_FALSE).toBe(false);
  }
  if (false && BRANCH_TRUE) {
    require("./branch-unused.js");
  } else {
    expect(require("./branch-logical-used.js")).toBe("logical-used");
  }
  if (true || BRANCH_FALSE) {
    expect(require("./branch-logical-used.js")).toBe("logical-used");
  } else {
    require("./branch-unused.js");
  }
  if ((BRANCH_TRUE || BRANCH_FALSE) && (BRANCH_FALSE || true) && (BRANCH_TRUE || false)) {
    expect(require("./branch-logical-used.js")).toBe("logical-used");
  } else {
    require("./branch-unused.js");
  }

  const unusedModule = "./branch-" + "unused.js";
  const usedModule = "./branch-" + "used.js";
  const logicalUsedModule = "./branch-" + "logical-used.js";
  const unusedFactory = `"${unusedModule}"(`;
  const usedFactory = `"${usedModule}"(`;
  const logicalUsedFactory = `"${logicalUsedModule}"(`;
  const unusedMarker = "__branchCondition" + "Unused";
  const usedMarker = "__branchCondition" + "Used";
  const logicalUsedMarker = "__branchCondition" + "LogicalUsed";
  expect(globalThis[unusedMarker]).toBe(undefined);
  expect(globalThis[usedMarker]).toBe(true);
  expect(globalThis[logicalUsedMarker]).toBe(true);
  expect(generated.includes(unusedFactory)).toBe(false);
  expect(generated.includes(usedFactory)).toBe(true);
  expect(generated.includes(logicalUsedFactory)).toBe(true);
  expect(generated.includes(unusedMarker)).toBe(false);
  expect(generated.includes(usedMarker)).toBe(true);
  expect(generated.includes(logicalUsedMarker)).toBe(true);
})
