import fs from "node:fs";
import path from "node:path";

/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  afterExecute(options) {
    for (const filename of ["main.js", "async.js"]) {
      const source = fs.readFileSync(
        path.join(options.output.path, filename),
        "utf-8",
      );
      const order = Array.from(
        source.matchAll(/module\.exports = "render-order:(\w+)"/g),
        (match) => match[1],
      );
      // ../loader.cjs!./loaded, ../z, ./a: sorting absolute identifiers
      // would put ./a before ../z instead.
      expect(order).toEqual(["loaded", "outside", "inside"]);
    }
  },
};
