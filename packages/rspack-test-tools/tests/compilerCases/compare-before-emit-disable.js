const path = require("path");
const fs = require("fs");
const rimraf = require("rimraf");

let first_asset_mtime;

/** @type {import('../../dist').TCompilerCaseConfig} */
module.exports = {
  description: "should write same content to same file",
  options(context) {
    return {
      output: {
        path: context.getDist(),
        filename: "main.js",
        compareBeforeEmit: false
      },
      context: context.getSource(),
      entry: "./d",
    };
  },
  async build(context, compiler) {
    rimraf.sync(context.getDist());
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
    expect(first_asset_mtime).not.toEqual(second_asset_mtime);
  }
};
