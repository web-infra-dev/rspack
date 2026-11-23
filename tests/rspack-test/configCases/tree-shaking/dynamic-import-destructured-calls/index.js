const fs = require("fs");
const path = require("path");

it("should not reference a receiver for destructured calls", async () => {
  const { missing, provided } = await import(/* webpackChunkName: "commonjs" */ "./lib.cjs");
  expect(missing?.()).toBeUndefined();
  expect(provided()).toBe("provided");
  const content = fs.readFileSync(path.join(__dirname, "commonjs.js"), "utf-8");
  expect(content).not.toContain("unused-cjs-marker");
});

it("should share contexts between destructured calls and reads", async () => {
  const page = ["home"].pop();
  const called = await import(`./pages/${page}.js`).then(({ default: init }) => init());
  const read = await import(`./pages/${page}.js`).then(({ default: init }) => init);
  const tagged = await import(`./pages/${page}.js`).then(({ default: init }) => init`value`);
  const { default: awaited } = await import(`./pages/${page}.js`);
  const [{ default: combined }] = await Promise.all([import(`./pages/${page}.js`)]);
  expect(called).toBe("home");
  expect(read()).toBe("home");
  expect(tagged).toBe("home");
  expect(awaited()).toBe("home");
  expect(combined()).toBe("home");
});

it("should not reference a receiver for nested destructured calls", async () => {
  const { nested: { missing } } = await import(/* webpackChunkName: "nested" */ "./nested-namespace.js");
  expect(missing?.()).toBeUndefined();
  const content = fs.readFileSync(path.join(__dirname, "nested.js"), "utf-8");
  expect(content).not.toContain("unused-esm-marker");
});

it("should retain receivers for member calls and tagged templates", async () => {
  const { receiver } = await import("./namespace.js");
  expect(receiver.call()).toBe("receiver-kept");
  expect(receiver.tag`value`).toBe("receiver-kept");
});
