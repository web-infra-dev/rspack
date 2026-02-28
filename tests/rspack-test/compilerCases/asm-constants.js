
/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  skip: true,
  description: "should not evaluate constants in asm.js",
  options(context) {
    return {
      entry: "./asmjs",
    };
  },
  async check({ files }) {
    expect(Object.keys(files)).toEqual(["/main.js"]);
    const bundle = files["/main.js"];
    expect(bundle).toMatch('"use asm";');
    expect(bundle).toMatch("101");
    expect(bundle).toMatch("102");
    expect(bundle).toMatch("103");
    expect(bundle).toMatch("104");
    expect(bundle).toMatch("105");
    expect(bundle).not.toMatch("106");
    expect(bundle).not.toMatch("107");
    expect(bundle).not.toMatch("108");
    expect(bundle).toMatch("109");
    expect(bundle).toMatch("110");
  }
};
