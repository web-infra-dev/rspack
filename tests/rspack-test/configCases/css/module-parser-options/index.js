import "./style.module.css";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should support css module parser feature toggles", () => {
  const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");

  expect(css).toContain("@keyframes fade--local");
  expect(css).toContain("animation-name: fade--local");
  expect(css).toContain("@counter-style thumbs--local");
  expect(css).toContain("list-style: thumbs--local");
  expect(css).toContain("@font-palette-values palette--local");
  expect(css).toContain("font-palette: palette--local");
  expect(css).toContain("@property brand-color--local");
  expect(css).toContain("var(brand-color--local)");
});
