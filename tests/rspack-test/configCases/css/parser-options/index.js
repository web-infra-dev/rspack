import * as animation from "./animation-name.module.css";
import * as styles from "./options.module.css";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

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
  expect(content).toContain("@counter-style thumbs");
  expect(content).toContain("list-style: thumbs");
});

it("should support disabling import and url handling", () => {
  const content = css();
  expect(content).toContain('@import "./imported.css";');
  expect(content).toContain('url("./missing.png")');
  expect(content).not.toContain(".imported");
});
