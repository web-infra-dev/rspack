it("should emit the Node provider's shared dependency IDs", async () => {
  const container = require("./container.js");
  await container.init({});
  const { default: shared, cache } = (await container.get("./Shared"))();
  const { id: payload } = (await container.get("./Payload"))();
  expect(cache.retainedModuleIds.map(String)).toEqual(
    expect.arrayContaining([String(shared.id), String(await shared.lazy())]),
  );
  expect(cache.retainedModuleIds.map(String)).not.toContain(String(payload));
  expect(cache.modules).toHaveProperty(String(payload));
  expect(Object.keys(container).sort()).toEqual(["get", "init"]);
});
