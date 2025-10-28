const { createFsFromVolume, Volume } = require("memfs");
const deprecationTracking = require("@rspack/test-tools/helper/legacy/deprecationTracking");
let tracker = null;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should set compiler.watching correctly",
  options(context) {
    return {
      entry: "./c",
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watching = compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
        watching.close(resolve);
      });
      expect(compiler.watching).toBe(watching);
    });
  },
}, {
  description: "should flag watchMode as true in watch",
  options(context) {
    return {
      entry: "./c",
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watch = compiler.watch({}, err => {
        if (err) return reject(err);
        expect(compiler.watchMode).toBeTruthy();
        watch.close(() => {
          expect(compiler.watchMode).toBeFalsy();
          resolve();
        });
      });
    });
  },
}, {
  description: "should deprecate when watch option is used without callback",
  options(context) {
    tracker = deprecationTracking.start();
    return {
      watch: true
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {

  },
  async check() {
    const deprecations = tracker();
    expect(deprecations).toHaveLength(1);
    expect(deprecations[0].message).toContain("A 'callback' argument needs to be provided");
  }
}];
