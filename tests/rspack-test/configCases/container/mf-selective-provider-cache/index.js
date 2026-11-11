it("should clear unshared exports but keep shared and async dependency identity", async () => {
  const container = require("./container.js");
  await container.init({});
  const load = async (key) => (await container.get(key))().default;
  const shared = await load("./Shared");
  // First lazy execution happens after clearing the provider cache.
  const payload = await load("./Payload");
  expect(typeof container.__webpack_clear_exposed_cache__).toBe("function");
  container.__webpack_clear_exposed_cache__();
  expect(await load("./Payload")).not.toBe(payload);
  expect(await load("./Shared")).toBe(shared);
  const lazy = await shared.lazy();
  container.__webpack_clear_exposed_cache__();
  expect(await shared.lazy()).toBe(lazy);
});
