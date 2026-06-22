import * as styles from "./style.module.css";

it("should preserve undeclared css module idents", () => {
  const fs = __non_webpack_require__("fs");
  const path = __non_webpack_require__("path");
  const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");

  expect(css).toMatchFileSnapshotSync(
    path.join(__SNAPSHOT__, "bundle0.css.txt")
  );
  expect(styles).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "exports.txt"));
});
