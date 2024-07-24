const fs = require("fs");
const path = require("path");

it("should tree shaking dynamic import await", async () => {
  const { a, b } = await import('./lib');
  expect(a).toBe("property-a");
  expect(b).toBe("property-b");
  const content = fs.readFileSync(path.join(path.dirname(__filename), 'chunk.js'));
  expect(content.includes("property-c")).toBe(false);
  expect(content.includes("property-d")).toBe(false);
});
