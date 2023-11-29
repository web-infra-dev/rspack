// @ts-nocheck
const { Tester, DiffProcessor, DiffStatsReporter } = require("../packages/rspack-test-tools/dist");
const rimraf = require("rimraf");
const path = require("path");
const fs = require("fs-extra");
const csvToMarkdown = require("csv-to-markdown-table");

process.env['RSPACK_DIFF'] = "true"; // 开启DIFF

const CASE_DIR = path.resolve(__dirname, '../diffcases');
const toPercent = d => (d * 100).toFixed(2) + '%';

(async () => {
  const cases = fs
    .readdirSync(CASE_DIR)
    .filter(testName => !testName.startsWith("."));

  const result = [];
  const reporter = new DiffStatsReporter({
    file: path.join(__dirname, '../diff_output')
  });

  while (cases.length) {
    const name = cases.shift();
    const source = path.join(CASE_DIR, name);
    try {
      const processor = new DiffProcessor({
        webpackPath: require.resolve('webpack'),
        rspackPath: require.resolve('../packages/rspack'),
        files: ['bundle.js', 'main.js'],
        modules: true,
        runtimeModules: true,
        ignoreModuleId: true,
        ignoreModuleArguments: true,
        ignorePropertyQuotationMark: true,
        onCompareModules: function (file, results) {
          reporter.increment(name, results);
        },
        onCompareRuntimeModules: function (file, results) {
          reporter.increment(name, results);
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
    } catch (e) {
      reporter.failure(name);
    }
  }
  await reporter.output();
})();

