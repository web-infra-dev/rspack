const fs = require("fs");
const path = require("path");

it("should tree shaking dynamic import await", async () => {
  const { a, b } = await import(/* webpackChunkName: "chunk" */ './lib');
  expect(a).toBe("property-a");
  expect(b).toBe("property-b");
  const content = fs.readFileSync(path.join(path.dirname(__filename), 'chunk.js'));
  expect(content.includes("property-c")).toBe(false);
  expect(content.includes("property-d")).toBe(false);
});

it("should tree shake a destructured namespace", async () => {
  const { ns: alias, receiver } = await import(/* webpackChunkName: "chunk" */ './namespace');
  expect(alias.a).toBe("property-a");
  expect(receiver.tag`value`).toBe("receiver-kept");
  const content = fs.readFileSync(path.join(path.dirname(__filename), 'chunk.js'), 'utf-8');
  expect(content).not.toContain("property-c");
});
