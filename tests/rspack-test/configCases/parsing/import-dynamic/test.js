const fs = require("fs");
const path  = require("path");

it("import() a dynamical expression should preserve as-is when `importDynamic` is disabled", () => {
  const code = fs.readFileSync(path.join(__dirname, "./bundle0.js"), 'utf-8');
  expect(code).not.toContain("import('./other.js')");
  expect(code).toContain("import('./' + dir + '/other.js')");
  expect(code).toContain("import(dir)");
});
