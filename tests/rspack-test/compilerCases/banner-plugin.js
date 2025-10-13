const rspack = require("@rspack/core");
const fs = require("fs");

let lastStats = null;
/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should cache assets",
  options(context) {
    return {
      mode: "development",
      entry: {
        entry1: context.getDist("entry1.js"),
        entry2: context.getDist("entry2.js")
      },
      output: {
        path: context.getDist("dist")
      },
      plugins: [new rspack.BannerPlugin("banner is a string")]
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = fs;
    fs.writeFileSync(context.getDist("entry1.js"), "1", "utf-8");
    fs.writeFileSync(context.getDist("entry2.js"), "1", "utf-8");
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);
        try {
          const footerFileResults = fs.readFileSync(context.getDist("dist/entry1.js"), "utf8").split("\n");
          expect(footerFileResults[0]).toBe("/*! banner is a string */");
          fs.writeFileSync(context.getDist("entry2.js"), "2", "utf-8");
          compiler.run((err, stats) => {
            if (err) return reject(err);
            lastStats = stats;
            resolve();
          }, {
            modifiedFiles: new Set([context.getDist("entry2.js")])
          });
        } catch (err) {
          reject(err);
        }

      });
    });
  },
  async check() {
    const { assets } = lastStats.toJson();
    expect(assets.find(as => as.name === "entry1.js").emitted).toBe(false);
    expect(assets.find(as => as.name === "entry2.js").emitted).toBe(true);
  }
}, {
  description: "can place banner as footer",
  options(context) {
    return {
      mode: "development",
      entry: {
        footerFile: context.getDist("footerFile.js")
      },
      output: {
        path: context.getDist("dist")
      },
      plugins: [
        new rspack.BannerPlugin({
          banner: "banner is a string",
          footer: true
        })
      ]
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = fs;
    fs.writeFileSync(context.getDist("footerFile.js"), "footer", "utf-8");
  },
  async check({ context }) {
    const footerFileResults = fs.readFileSync(context.getDist("dist/footerFile.js"), "utf8").split("\n");
    expect(footerFileResults.pop()).toBe("/*! banner is a string */");
  }
}];
