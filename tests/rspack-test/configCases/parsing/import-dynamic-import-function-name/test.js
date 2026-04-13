const fs = require("fs");
const path = require("path");

it("should respect importFunctionName when `importDynamic` is disabled", () => {
  const code = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
  expect(code).not.toContain("import('./other.js')");
  expect(code).not.toContain("import('./' + dir + '/other.js')");
  expect(code).not.toContain("import(dir)");
  expect(code).toContain("__import__('./' + dir + '/other.js')");
  expect(code).toContain("__import__(dir)");
});
