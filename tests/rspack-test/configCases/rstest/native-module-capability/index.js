const fs = require("node:fs");
const path = require("node:path");
const metadata = JSON.parse(
  fs.readFileSync(path.join(__dirname, "capability.json"), "utf8"),
);
const capability = {
  kind: "require",
  origin: "source",
  parts: [],
  unsupported: "native-module-capability",
};
const byResource = new Map(
  metadata.modules.map((module) => [module.resource, module.metadata]),
);

it("reports native loader acquisition without enabling createRequire parsing", () => {
  expect(metadata.createRequire).toBe(false);
  for (const resource of [
    "named.js",
    "namespace.js",
    "default.js",
    "commonjs.cjs",
    "folded.cjs",
    "dynamic.js",
    "factory.js",
    "benign.js",
  ]) {
    expect(byResource.get(resource)).toEqual({
      version: 1,
      calls: [capability],
    });
  }
  expect(byResource.get("ordinary.js")).toEqual({ version: 1, calls: [] });
  expect(byResource.get("resolve-only.js")).toEqual({
    version: 1,
    calls: [{ kind: "resolve", origin: "source", parts: ["node:module"] }],
  });
  expect(
    metadata.modules.some((module) => module.resource === "provider.cjs"),
  ).toBe(false);
});

it("preserves native aliases, namespace access, destructuring, and dynamic import execution", async () => {
  const runtime = require(path.join(__dirname, "capability.js"));
  await expect(runtime.readAll()).resolves.toEqual([
    42,
    42,
    42,
    42,
    42,
    42,
    42,
    true,
    "value",
  ]);
});
