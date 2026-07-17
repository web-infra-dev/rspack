import { getValue as getRootAValue } from "./root-a";
import { stable } from "./root-shared";

it("should keep runtime-specific concatenation results isolated", () => {
  expect(getRootAValue()).toBe(42);
  expect(stable).toBe(1);

  const rootAGroup = __STATS__.modules.find(module =>
    module.modules?.some(nested => nested.name.endsWith("root-a.js"))
  );
  expect(rootAGroup).toBeDefined();

  const shared = __STATS__.modules.find(module => module.name.endsWith("root-shared.js"));
  expect(shared).toBeDefined();
  // Runtime-mode compilation exercises the runtime-keyed bailout; the default compilation
  // exercises the same candidate across incompatible root chunks.
  const leafWasConcatenated = rootAGroup.modules.some(module => module.name.endsWith("leaf.js"));
  const expectedBailout = leafWasConcatenated
    ? "runtime-dependent referenced"
    : "not in the same chunk(s)";
  expect(shared.optimizationBailout).toEqual(
    expect.arrayContaining([expect.stringContaining(expectedBailout)])
  );
});
