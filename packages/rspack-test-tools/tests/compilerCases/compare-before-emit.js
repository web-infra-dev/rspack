const path = require("path");
const fs = require("fs");
const { rimrafSync } = require("rimraf");

let first_asset_mtime;

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
  description: "should write emit same content to same file",
  options(context) {
    return {
      output: {
        path: context.getDist(),
        filename: "main.js",
      },
      context: context.getSource(),
      entry: "./d",
    };
  },
  async build(context, compiler) {
    rimrafSync(context.getDist());
    await new Promise(resolve => {
      compiler.run(() => {
        first_asset_mtime = fs.statSync(path.join(context.getDist("main.js")))?.mtime;
        compiler.run(() => {
          resolve();
        });
      });
    });
  },
  async check(context) {
    let second_asset_mtime = fs.statSync(path.join(context.getDist("main.js")))?.mtime;
    expect(first_asset_mtime).toEqual(second_asset_mtime);
  }
};
