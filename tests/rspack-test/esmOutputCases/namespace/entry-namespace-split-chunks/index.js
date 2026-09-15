export * as movedNs from "./moved-target";
export * as mergedNs from "./merged-target";
export * as leakyNs from "./leaky-target";
export * as alphaNs from "./alpha-target";
export * as betaNs from "./beta-target";
export { sharedValue } from "./leaky-shared";
export { unrelated } from "./unrelated";

it("allows later splitChunks and falls back when the chunk signature changes", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.movedNs.value).toBe("moved");
  expect(mod.mergedNs.value).toBe("merged");
  expect(mod.leakyNs.value).toBe("shared");
  expect(Object.keys(mod.alphaNs)).toEqual(["foo"]);
  expect(mod.alphaNs.foo).toBe("alpha");
  expect(Object.keys(mod.betaNs)).toEqual(["value"]);
  expect(mod.betaNs.value).toBe("beta");
  expect("sharedValue" in mod.leakyNs).toBe(false);
  expect(mod.sharedValue).toBe("shared");
  expect(mod.unrelated).toBe("unrelated");
  expect(globalThis.entryNamespaceSplitOrder).toEqual(["moved", "merged", "leaky"]);
  delete globalThis.entryNamespaceSplitOrder;
});
