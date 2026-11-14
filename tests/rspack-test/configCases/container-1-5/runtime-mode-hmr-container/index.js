it("initializes federation on the shared runtime context with HMR enabled", async () => {
  const container = require("./container-file.js");
  await container.init({});
  const factory = await container.get("./value");
  expect(factory()).toBe(42);
});
