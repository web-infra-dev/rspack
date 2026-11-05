import path from "node:path";

export default function (code, map) {
  // Runs directly after builtin:lightningcss-loader, before Rspack parses or sizes the module.
  expect(code).not.toContain("@custom-media");
  expect(code).not.toContain("--");
  expect(code).toContain("@media (width >= 800px)");
  expect(code).toContain("@media (width >= 800px) and (orientation: landscape)");
  expect(code).toContain("@media (width <= 500px)");
  expect(code).toContain("@media (width >= 900px)");
  expect(code).not.toContain("1200px");
  // `minify: false` keeps formatted output.
  expect(code).toContain("\n  .wide {\n    color: red;\n  }");

  expect(map).toBeTruthy();
  expect(map.sources).toEqual([
    path.resolve(import.meta.dirname, "index.module.css").replace(/\\/g, "/"),
  ]);

  this.callback(null, code, map);
}
