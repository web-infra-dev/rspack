// @ts-nocheck
const { Tester, DiffProcessor, DiffStatsReporter, DiffHtmlReporter } = require("../packages/rspack-test-tools/dist");
const rimraf = require("rimraf");
const path = require("path");
const fs = require("fs-extra");

process.env['RSPACK_DIFF'] = "true"; // enable rspack diff injection

const CASE_DIR = path.resolve(__dirname, '../diffcases');
const OUTPUT_DIR = path.join(__dirname, '../diff_output');

(async () => {
  const cases = fs
    .readdirSync(CASE_DIR)
    .filter(testName => !testName.startsWith("."));

  rimraf.sync(OUTPUT_DIR);

  const statsReporter = new DiffStatsReporter({
    file: path.join(OUTPUT_DIR, 'stats.md'),
    report: true,
  });
  const htmlReporter = new DiffHtmlReporter({
    dist: OUTPUT_DIR,
    ignore: {
      test: () => false,
    },
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
        ignoreBlockOnlyStatement: true,
        ignoreSwcHelpersPath: true,
        ignoreObjectPropertySequence: true,
        ignoreCssFilePath: true,
        detail: false,
        onCompareModules: function (file, results) {
          htmlReporter.increment(name, results);
          statsReporter.increment(name, results);
        },
        onCompareRuntimeModules: function (file, results) {
          htmlReporter.increment(name, results);
          statsReporter.increment(name, results);
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
        await tester.check({ expect: () => ({ toBe: (() => {}) }) });
      } while (tester.next());
      await tester.resume();
    } catch (e) {
      htmlReporter.failure(name)
      statsReporter.failure(name);
      console.error(e);
    }
  }
  await htmlReporter.output();
  await statsReporter.output();
})();

