import { createFsFromVolume, Volume } from "memfs";

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
export default {
  description: "should create JSON of children stats",
  options(context) {
    return [
      {
        context: import.meta.dirname,
        entry: "../fixtures/a"
      },
      {
        context: import.meta.dirname,
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