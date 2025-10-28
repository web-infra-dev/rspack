
/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should compile a file with multiple chunks",
  options(context) {
    return {
      entry: "./chunks",
    };
  },

  async check({ files, stats }) {
    expect(stats.chunks).toHaveLength(2);
    expect(Object.keys(files).sort().reverse()).toEqual(["/main.js", "/555.js"]); // CHANGE
    const bundle = files["/main.js"];
    const chunk = files["/555.js"]; // CHANGE
    expect(bundle).toMatch("function __webpack_require__(");
    expect(bundle).toMatch("__webpack_require__(/*! ./b */");
    expect(chunk).not.toMatch("__webpack_require__(/* ./b */");
    expect(bundle).toMatch("./chunks.js");
    expect(chunk).toMatch("./a.js");
    expect(chunk).toMatch("./b.js");
    expect(chunk).toMatch("This is a");
    expect(bundle).not.toMatch("This is a");
    expect(chunk).toMatch("This is b");
    expect(bundle).not.toMatch("This is b");
    expect(bundle).not.toMatch("4: function(");
    expect(bundle).not.toMatch("fixtures");
    expect(chunk).not.toMatch("fixtures");
    expect(bundle).toMatch("webpackChunk");
    expect(chunk).toMatch('self["webpackChunk"] || []).push');
  }
};
