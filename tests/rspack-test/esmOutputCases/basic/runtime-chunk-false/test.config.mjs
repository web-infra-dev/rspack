import fs from "node:fs";
import path from "node:path";

export default {
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, "main.mjs"),
      "utf-8",
    );

    expect(source).not.toContain("export { __rspack_");
  },
};
