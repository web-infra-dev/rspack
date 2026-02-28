const rspack = require("@rspack/core");
const path = require("path");
const total = 100;
const runnings = [];
let finish = 0;

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [{
  description: "should work well in multiCompilers",
  options(context) {
    return Array(total).fill({
      context: path.join(__dirname, "fixtures"),
      entry: "./a.js"
    });
  }
}, {
  description: "should work well in concurrent",
  async build(context, compiler) {
    for (let i = 0; i < total; i++) {
      if (i % 10 == 0) {
        // Insert new instance while we are running
        rspack(
          {
            context: path.join(__dirname, "fixtures"),
            entry: "./a.js"
          },
          () => { }
        );
      }

      runnings.push(
        new Promise(resolve => {
          rspack(
            {
              context: path.join(__dirname, "fixtures"),
              entry: "./a.js"
            },
            err => {
              resolve(null);
              if (!err) finish++;
            }
          );
        })
      );
    }

    await Promise.all(runnings);
    expect(finish).toBe(total);
  }
}];