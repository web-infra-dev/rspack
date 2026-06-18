import * as animation from "./animation-name.module.css";
import * as externalToken from "./external-token.module.css";
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

it("should not rename undeclared custom property references by default", () => {
  expect(externalToken.card).toBe("external-token.module_module_css-card");

  const content = css();
  expect(content).toContain("animation-name: externalFade");
  expect(content).toContain(
    "background-image: --external-function(var(--external-bg))"
  );
  expect(content).toContain("background-color: var(--external-bg)");
  expect(content).toContain("border-radius: var(--external-radius)");
  expect(content).toContain("font-palette: --external-palette");
  expect(content).toContain("grid-area: externalArea");
  expect(content).toContain("grid-row: externalRow");
  expect(content).toContain("list-style: externalCounter");
  expect(content).toContain("@container externalContainer (min-width: 400px)");
  expect(content).toContain("--external-token\\.module_module_css-local-color: red");
  expect(content).toContain(
    "color: var(--external-token\\.module_module_css-local-color)"
  );
  expect(content).not.toContain("externalFade_module_css");
  expect(content).not.toContain("externalCounter_module_css");
  expect(content).not.toContain("externalContainer_module_css");
  expect(content).not.toContain("externalArea_module_css");
  expect(content).not.toContain("--external-function_module_css");
  expect(content).not.toContain("--external-palette_module_css");
  expect(content).not.toContain("--external-bg_module_css");
  expect(content).not.toContain("--external-radius_module_css");
});

it("should support disabling import and url handling", () => {
  const content = css();
  expect(content).toContain('@import "./imported.css";');
  expect(content).toContain('url("./missing.png")');
  expect(content).not.toContain(".imported");
});
