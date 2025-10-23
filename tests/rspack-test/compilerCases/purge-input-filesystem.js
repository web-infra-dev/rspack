/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [(() => {
  const mockPurge = rstest.fn();
  return {
    description: "invokes purge() if inputFileSystem.purge",
    options(context) {
      return {
        entry: "./c",
      };
    },
    async compiler(context, compiler) {
      compiler.inputFileSystem = {
        purge: mockPurge
      };
      compiler.purgeInputFileSystem();
    },
    async check() {
      expect(mockPurge.mock.calls.length).toBe(1);
    }
  }
})(), (() => {
  const mockPurge = rstest.fn();
  return {
    description: "does NOT invoke purge() if !inputFileSystem.purge",
    options(context) {
      return {
        entry: "./c",
      };
    },
    async compiler(context, compiler) {
      compiler.inputFileSystem = null;
      compiler.purgeInputFileSystem();
    },
    async check() {
      expect(mockPurge.mock.calls.length).toBe(0);
    }
  };
})()];
