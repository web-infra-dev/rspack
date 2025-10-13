const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = {
  description: "should create JSON of children stats",
  options(context) {
    return [
      {
        context: __dirname,
        entry: "../fixtures/a"
      },
      {
        context: __dirname,
        entry: "../fixtures/b"
      }
    ];
  },
  compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);
        try {
          const statsObject = stats.toJson();
          expect(statsObject).toEqual(
            expect.objectContaining({ children: expect.any(Array) })
          );
          expect(statsObject.children).toHaveLength(2);
          resolve();
        } catch (e) {
          reject(e);
        }
      });
    });
  }
};