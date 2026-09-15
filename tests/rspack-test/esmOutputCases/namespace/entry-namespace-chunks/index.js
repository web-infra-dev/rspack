export * as zDirectNs from "./target-direct";
import * as imported from "./target-imported";
export { imported as aImportedNs };

import * as observed from "./target-observed";
import { sharedValue, trace } from "./shared";

export { observed as observedNs };
export const observedKeys = Object.keys(observed);
export { directOnly } from "./target-direct";
export { foo as collision } from "./target-direct";
export { counter, increment } from "./target-direct";
export const entryShared = sharedValue;

it("should preserve entry namespace exports across target chunks", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  expect(trace).toEqual(["direct", "imported"]);

  expect(mod.zDirectNs.foo).toBe("shared:direct-private");
  expect(mod.zDirectNs.bar).toBe("DROP_BAR_SENTINEL");
  expect(mod.zDirectNs.collision).toBe("target-collision");
  expect(Object.keys(mod.zDirectNs)).toEqual(["bar", "collision", "counter", "directOnly", "foo", "increment"]);
  expect(mod.counter).toBe(0);
  mod.increment();
  expect(mod.zDirectNs.counter).toBe(1);
  expect(mod.counter).toBe(1);
  expect(mod.aImportedNs.foo).toBe("shared:imported-private");
  expect(mod.aImportedNs.bar).toBe("DROP_IMPORTED_BAR_SENTINEL");
  expect(mod.aImportedNs.default).toBe(42);
  expect(Object.keys(mod.aImportedNs)).toEqual(["bar", "default", "foo"]);
  expect(mod.observedKeys).toEqual(["value"]);
  expect(Object.keys(mod.observedNs)).toEqual(["value"]);
  expect(mod.observedNs.value).toBe("observed");
  expect(mod.directOnly).toBe("direct-only");
  expect(mod.collision).toBe("shared:direct-private");
  expect(mod.entryShared).toBe("shared");
});
