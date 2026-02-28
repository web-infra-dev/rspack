/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  skip: true,
  description: "returns booleanized this.parentCompilation",
  options(context) {
    return {
      entry: "./c",
    };
  },
  async compiler(context, compiler) {
    compiler.parentCompilation = "stringyStringString";
    const response1 = compiler.isChild();
    expect(response1).toBe(true);

    compiler.parentCompilation = 123456789;
    const response2 = compiler.isChild();
    expect(response2).toBe(true);

    compiler.parentCompilation = {
      what: "I belong to an object"
    };
    const response3 = compiler.isChild();
    expect(response3).toBe(true);

    compiler.parentCompilation = ["Array", 123, true, null, [], () => { }];
    const response4 = compiler.isChild();
    expect(response4).toBe(true);

    compiler.parentCompilation = false;
    const response5 = compiler.isChild();
    expect(response5).toBe(false);

    compiler.parentCompilation = 0;
    const response6 = compiler.isChild();
    expect(response6).toBe(false);

    compiler.parentCompilation = null;
    const response7 = compiler.isChild();
    expect(response7).toBe(false);

    compiler.parentCompilation = "";
    const response8 = compiler.isChild();
    expect(response8).toBe(false);

    compiler.parentCompilation = NaN;
    const response9 = compiler.isChild();
    expect(response9).toBe(false);
  },
};
