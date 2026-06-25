import "./style.module.css";

const fs = require("fs");
const path = require("path");

it("should support css module parser feature toggles", () => {
  const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");

  expect(css).toContain("@keyframes fade--local");
  expect(css).toContain("animation-name: fade--local");
  expect(css).toContain("container-name: summary--local");
  expect(css).toContain("container: card--local / inline-size");
  expect(css).toContain("@container summary--local (min-width: 400px)");
  expect(css).toContain("@counter-style thumbs--local");
  expect(css).toContain("list-style: thumbs--local");
  expect(css).toContain("@font-palette-values palette--local");
  expect(css).toContain("font-palette: palette--local");
  expect(css).toContain("@function --transparent--local(--color, --alpha)");
  expect(css).toContain("--transparent--local(var(--brand-color--local), 0.8)");
  expect(css).toContain('"header--local header--local"');
  expect(css).toContain('"sidebar--local main--local"');
  expect(css).toContain("grid-area: header--local");
  expect(css).toContain("grid-row: sidebar--local");
  expect(css).toContain("@property --brand-color--local");
  expect(css).toContain("var(--brand-color--local)");
});
