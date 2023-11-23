const { Tester, DiffProcessor } = require("../packages/rspack-test-tools");
const rimraf = require("rimraf");
const path = require("path");
const fs = require("fs-extra");

process.env['RSPACK_DIFF'] = "true"; // 开启DIFF

(async () => {
  // const cases = fs
  //   .readdirSync(__dirname)
  //   .filter(testName => !testName.startsWith(".") && !['extract-license'].includes(testName));

  const cases = ["arco-pro"];
  const total = {
    invalidCases: 0,
    validCases: 0,
    totalLines: 0,
    sameLines: 0,
    diffLines: 0,
    totalModules: 0,
    sameModules: 0,
    diffModules: 0
  };

  while (cases.length) {
    const source = path.join(__dirname, cases.shift());
    try {
      console.log(`start processing: ${source}`);
      let hasFileCompared = false;
      const processor = new DiffProcessor({
        webpackPath: require.resolve('webpack'),
        rspackPath: require.resolve('../packages/rspack'),
        files: ['bundle.js', 'main.js'],
        modules: true,
        runtimeModules: true,
        ignoreModuleId: true,
        ignoreModuleArguments: true,
        ignorePropertyQuotationMark: true,
        onCompareFile: function (file, result) {
          console.log(file, result.type);
          if (['different'].includes(result.type)) {
            hasFileCompared = true;
          }
        },
        onCompareModules: function (file, results) {
          total.totalModules += results.length;
          total.sameModules += results.filter(r => r.type === 'same').length;
          total.diffModules += results.filter(r => r.type !== 'same').length;

          total.totalLines += results.reduce((l, r) => l + r.lines.common + r.lines.source + r.lines.dist, 0);
          total.sameLines += results.reduce((l, r) => l + r.lines.common, 0);
          total.diffLines += results.reduce((l, r) => l + r.lines.source + r.lines.dist, 0);
        },
        onCompareRuntimeModules: function (file, results) {
          total.totalModules += results.length;
          total.sameModules += results.filter(r => r.type === 'same').length;
          total.diffModules += results.filter(r => r.type !== 'same').length;

          total.totalLines += results.reduce((l, r) => l + r.lines.common + r.lines.source + r.lines.dist, 0);
          total.sameLines += results.reduce((l, r) => l + r.lines.common, 0);
          total.diffLines += results.reduce((l, r) => l + r.lines.source + r.lines.dist, 0);
        },
      });

      const tester = new Tester({
        name: 'test',
        src: source,
        dist: path.join(source, "dist"),
        steps: [processor]
      });

      rimraf.sync(path.join(source, "dist"));
      await tester.prepare();
      do {
        await tester.compile();
        await tester.check();
      } while (tester.next());
      await tester.resume();
      if (hasFileCompared) {
        total.validCases++;
      } else {
        total.invalidCases++;
      }
    } catch (e) {
      total.invalidCases++;
    }
    console.log(total);
  }
})();

