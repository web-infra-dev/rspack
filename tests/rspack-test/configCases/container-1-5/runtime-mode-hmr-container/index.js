it("initializes federation on the loader with HMR enabled", async () => {
  const container = require("./container-file.js");
  await container.init({});
  const factory = await container.get("./value");
  const { value, getLoader } = factory();
  expect(value).toBe(42);
  const loader = getLoader();
  expect(loader.federation.instance).toBeDefined();
  expect(loader.initContainer).toBe(container.init);
  expect(loader.getContainer).toBe(container.get);
});
