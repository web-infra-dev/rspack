const mod = require("./module");
const collectedShares = __non_webpack_require__('./share-usage.json')

// Actually use the exports so they're marked as used
const locallyUsed = mod.used;
const alsoLocallyUsed = mod.alsoUsed;
const usedInBothPlaces = mod.usedBoth;

it("should tree shake unused exports in shared modules", () => {
  // Directly used exports - should be available
  expect(locallyUsed).toBe(42);
  expect(alsoLocallyUsed).toBe("directly imported");
  expect(usedInBothPlaces).toBe("used locally and externally");

  // Unused exports - should be tree-shaken (undefined)
  expect(mod.unused).toBe(undefined);
  expect(mod.alsoUnused).toBe(undefined);
  expect(mod.neverUsed).toBe(undefined);

  // Externally preserved exports (via external-usage.json) - should be available
  expect(mod.externallyUsed1).toBe("preserved for remote-app");
  expect(mod.externallyUsed2).toBe("preserved for analytics");
  expect(typeof mod.sharedUtility).toBe("function");
  expect(mod.sharedUtility()).toBe("external system needs this");

  // Assert share-usage.json correctly tracks local usage
  expect(collectedShares.treeShake.module).toBeDefined();
  
  // Locally used exports should be marked as true
  expect(collectedShares.treeShake.module.used).toBe(true);
  expect(collectedShares.treeShake.module.alsoUsed).toBe(true);
  expect(collectedShares.treeShake.module.usedBoth).toBe(true);
  
  // Locally unused exports should be marked as false (even if preserved externally)
  expect(collectedShares.treeShake.module.unused).toBe(false);
  expect(collectedShares.treeShake.module.alsoUnused).toBe(false);
  expect(collectedShares.treeShake.module.neverUsed).toBe(false);
  expect(collectedShares.treeShake.module.externallyUsed1).toBe(false);
  expect(collectedShares.treeShake.module.externallyUsed2).toBe(false);
  expect(collectedShares.treeShake.module.sharedUtility).toBe(false);
});
