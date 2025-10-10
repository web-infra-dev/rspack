const path = require("path");
const fs = require("fs");
const { createFsFromVolume, Volume } = require("memfs");

function createTestCase(changeTimeout, invalidate) {
  const fixturePath = path.join(__dirname, "../fixtures", "temp-" + changeTimeout);
  const filePath = path.join(fixturePath, "file.js");
  const file2Path = path.join(fixturePath, "file2.js");
  const loaderPath = path.join(__dirname, "../fixtures", "delay-loader.js");
  const memfs = createFsFromVolume(new Volume());

  return {
    description: `time between changes ${changeTimeout}ms${invalidate ? " with invalidate call" : ""}`,
    options(context) {
      try {
        fs.mkdirSync(fixturePath);
      } catch (e) {
        // empty
      }
      fs.writeFileSync(filePath, "require('./file2')", "utf-8");
      fs.writeFileSync(file2Path, "original", "utf-8");
      return {
        mode: "development",
        entry: loaderPath + "!" + filePath,
        output: {
          path: "/directory",
          filename: "bundle.js"
        }
      };
    },
    async compiler(context, compiler) {
      compiler.outputFileSystem = memfs;
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        try {
          let onChange;
          compiler.hooks.done.tap("WatchDetectionTest", () => {
            try {
              if (onChange) onChange();
            } catch (e) {
              console.error(e);
            }
          });
          let watcher;

          step1();

          function step1() {
            onChange = () => {
              if (
                memfs.readFileSync("/directory/bundle.js") &&
                memfs
                  .readFileSync("/directory/bundle.js")
                  .toString()
                  .indexOf("original") >= 0
              )
                step2();
            };

            watcher = compiler.watch(
              {
                aggregateTimeout: 50
              },
              () => { }
            );
          }

          function step2() {
            onChange = () => {
              expect(compiler.modifiedFiles).not.toBe(undefined);
              expect(compiler.removedFiles).not.toBe(undefined);
            };

            fs.writeFile(
              filePath,
              "require('./file2'); again",
              "utf-8",
              handleError
            );

            setTimeout(step3, changeTimeout);
          }

          function step3() {
            if (invalidate) watcher.invalidate();
            fs.writeFile(file2Path, "wrong", "utf-8", handleError);

            setTimeout(step4, changeTimeout);
          }

          function step4() {
            onChange = () => {
              expect(compiler.modifiedFiles).not.toBe(undefined);
              expect(compiler.removedFiles).not.toBe(undefined);
              if (
                memfs
                  .readFileSync("/directory/bundle.js")
                  .toString()
                  .indexOf("correct") >= 0
              )
                step5();
            };

            fs.writeFile(file2Path, "correct", "utf-8", handleError);
          }

          function step5() {
            onChange = null;

            watcher.close(() => {
              setTimeout(resolve, 100);
            });
          }

          function handleError(err) {
            if (err) reject(err);
          }
        } catch (e) {
          console.error(e);
          reject(e);
        }
      });
    },
    check() {
      try {
        fs.unlinkSync(filePath);
      } catch (e) {
        // empty
      }
      try {
        fs.unlinkSync(file2Path);
      } catch (e) {
        // empty
      }
      try {
        fs.rmdirSync(fixturePath);
      } catch (e) {
        // empty
      }
    }
  };
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  createTestCase(500, true),
  createTestCase(500, false),
  createTestCase(10, true),
  createTestCase(10, false)
];