import * as animation from "./animation-name.module.css";
import * as styles from "./options.module.css";

const fs = require("fs");
const path = require("path");

const css = () =>
  fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");

it("should support disabling animation renaming", () => {
  expect(Object.keys(animation)).not.toContain("animationName");

  const content = css();
  expect(content).toContain("animation: 3s animationName");
  expect(content).toContain("animation: animationName 3s");
  expect(content).toContain("animation-name: animationName");
  expect(content).toContain("@keyframes animationName");
});

it("should support disabling dashed and custom identifier renaming", () => {
  expect(styles.a).toBe("options.module_module_css-a");

  const content = css();
  expect(content).toContain("--brand-color: red");
  expect(content).toContain("color: var(--brand-color)");
  expect(content).toContain("container-name: summary");
  expect(content).toContain("container: card / inline-size");
  expect(content).toContain("@container summary (min-width: 400px)");
  expect(content).toContain("@counter-style thumbs");
  expect(content).toContain("--transparent(var(--brand-color), 0.8)");
  expect(content).toContain("@function --transparent(--color, --alpha)");
  expect(content).toContain('"header header"');
  expect(content).toContain('"sidebar main"');
  expect(content).toContain("grid-area: header");
  expect(content).toContain("grid-row: sidebar");
  expect(content).toContain("list-style: thumbs");
});

it("should support disabling import and url handling", () => {
  const content = css();
  expect(content).toContain('@import "./imported.css";');
  expect(content).toContain('url("./missing.png")');
  expect(content).not.toContain(".imported");
});
