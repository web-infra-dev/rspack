const fs = require("fs");
const path  = require("path");

it("`require.resolve` should preserve as-is when `requireResolve` is disabled", () => {
  const code = fs.readFileSync(path.join(__dirname, "./bundle0.js"), 'utf-8');
  expect(code).toContain("require.resolve(dir)");
  expect(code).toContain("require.resolve('./other.js')");
  expect(code).toContain("require.resolve('./foo/' + dir + '.js')");
  expect(code).toContain("require.resolve(process.env.RANDOM ? './foo/' + dir + '.js' : './bar/' + dir + 'js')");
  expect(code).toContain("require.resolve(external_path_default().resolve(__dirname, './other.js'))");
  expect(code).toContain("require.resolve('./a', { paths: [ cwd, external_path_default().resolve(cwd, 'node_modules') ] })");
});
