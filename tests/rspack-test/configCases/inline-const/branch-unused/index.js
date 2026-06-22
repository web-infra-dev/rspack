import { BRANCH_FALSE, BRANCH_TRUE, INNER, OUTER } from "./constants";

const fs = require("fs");
const path = require("path");

function emittedSource() {
  return fs
    .readdirSync(path.dirname(__filename))
    .filter(file => file.endsWith(".js"))
    .map(file => fs.readFileSync(path.join(path.dirname(__filename), file), "utf-8"))
    .join("\n");
}

it("should drop inactive branch dependencies guarded by inlined imported booleans", () => {
  const values = [];
  if (BRANCH_TRUE) {
    values.push("true-branch");
  } else {
    require("./branch-unused.js");
  }
  if (BRANCH_FALSE) {
    require("./branch-unused.js");
  } else {
    values.push("false-branch-alt");
  }
  if (BRANCH_FALSE) {
    import("./branch-unused.js");
  } else {
    values.push("dynamic-branch-alt");
  }

  expect(values).toEqual(["true-branch", "false-branch-alt", "dynamic-branch-alt"]);
  const unusedMarker = "unreachable: inline-const " + "branch-unused";
  expect(emittedSource().includes(unusedMarker)).toBe(false);
});

it("should drop nested inactive require.ensure block dependencies", async () => {
  if (OUTER) {
    await new Promise((resolve, reject) => {
      require.ensure([], require => {
        try {
          if (INNER) {
            require("./nested-unused");
          } else {
            const unusedMarker = "unreachable: nested require.ensure " + "branch-unused";
            expect(emittedSource().includes(unusedMarker)).toBe(false);
          }
          resolve();
        } catch (error) {
          reject(error);
        }
      });
    });
  } else {
    throw new Error("outer guard should be true");
  }
});
