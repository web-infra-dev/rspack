const fs = require("node:fs");
const path = require("node:path");
const { EventEmitter } = require("node:events");

it("should keep named module externals valid after merging async chunks", async () => {
  const source = fs.readFileSync(path.resolve(__STATS__.outputPath, "main.mjs"), "utf-8");

  expect(source).not.toMatch(/extends\s+[A-Za-z_$][\w$]*\.a\b/);

  const { default: Foo } = await import("./a.js");
  const instance = new Foo();

  expect(instance).toBeInstanceOf(EventEmitter);
});
