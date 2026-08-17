import * as styles from "./style.module.css";

const fs = require("fs");
const path = require("path");

it("should distinguish global keyword from a quoted module request", () => {
  expect(styles.keyword.endsWith(" external")).toBe(true);
  expect(styles.request.endsWith(" external")).toBe(false);
  expect(styles.request.split(" ")).toHaveLength(2);
});

it("should use the same localized name for ----a declarations and usages", () => {
  const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
  const declaration = css.match(/(--[^:\s]+): red/);
  expect(declaration).not.toBeNull();
  expect(css).toContain(`var(${declaration[1]})`);
});
