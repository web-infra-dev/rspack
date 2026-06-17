function MyApp() {
  return <div>Hello World</div>;
}

it("should emit react compiler output with swc-loader", () => {
  const fs = require("fs");
  const source = fs.readFileSync(__filename, "utf-8");
  expect(source).toContain(["react", "compiler-runtime"].join("/"));
  expect(source).toContain(["react", "memo_cache_sentinel"].join("."));
});
