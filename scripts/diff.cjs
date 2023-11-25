// @ts-nocheck
const { Tester, DiffProcessor } = require("../packages/rspack-test-tools/dist");
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

  while (cases.length) {
    const name = cases.shift();
    const source = path.join(CASE_DIR, name);
    const output = [`### ${name}`];
    const summary = {
      success: false,
      rspackOnlyRuntimeModules: 0,
      commonRuntimeModules: 0,
      webpackOnlyRuntimeModules: 0,
      rspackOnlyRuntimeLines: 0,
      commonRuntimeLines: 0,
      webpackOnlyRuntimeLines: 0,
      rspackOnlyRuntimeLinesInCommonModules: 0,
      commonRuntimeLinesInCommonModules: 0,
      webpackOnlyRuntimeLinesInCommonModules: 0,

      rspackOnlyModules: 0,
      commonModules: 0,
      webpackOnlyModules: 0,
      rspackOnlyLines: 0,
      commonLines: 0,
      webpackOnlyLines: 0,
      rspackOnlyLinesInCommonModules: 0,
      commonLinesInCommonModules: 0,
      webpackOnlyLinesInCommonModules: 0,
    };
    try {
      let hasFileCompared = false;
      const handleCompareModules = (type, results) => {
        const commonModules = results.filter(r => r.type === 'same' || r.type === 'different');
        summary[`rspackOnly${type}Modules`] += results.filter(r => r.type === 'only-source').length;
        summary[`webpackOnly${type}Modules`] += results.filter(r => r.type === 'only-webpack').length;
        summary[`common${type}Modules`] += commonModules.length;

        summary[`common${type}Lines`] += results.reduce((l, r) => l + r.lines.common, 0);
        summary[`rspackOnly${type}Lines`] += results.reduce((l, r) => l + r.lines.source, 0);
        summary[`webpackOnly${type}Lines`] += results.reduce((l, r) => l + r.lines.dist, 0);

        summary[`common${type}LinesInCommonModules`] += commonModules.reduce((l, r) => l + r.lines.common, 0);
        summary[`rspackOnly${type}LinesInCommonModules`] += commonModules.reduce((l, r) => l + r.lines.source, 0);
        summary[`webpackOnly${type}LinesInCommonModules`] += commonModules.reduce((l, r) => l + r.lines.dist, 0);
      };
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
          if (['different', 'same'].includes(result.type)) {
            summary.success = true;
          }
        },
        onCompareModules: function (file, results) {
          handleCompareModules('', results);
        },
        onCompareRuntimeModules: function (file, results) {
          handleCompareModules('Runtime', results);
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
      summary.success = false;
    }
    const tablify = (type) => {
      return csvToMarkdown(
        `${type || 'Normal'} Modules,Rspack Only,Common,Webpack Only,Common Percent
Modules,${summary[`rspackOnly${type}Modules`]},${summary[`common${type}Modules`]},${summary[`webpackOnly${type}Modules`]},${toPercent(summary[`common${type}Modules`] / (summary[`rspackOnly${type}Modules`] + summary[`webpackOnly${type}Modules`] + summary[`common${type}Modules`]))}
Lines,${summary[`rspackOnly${type}Lines`]},${summary[`common${type}Lines`]},${summary[`webpackOnly${type}Lines`]},${toPercent(summary[`common${type}Lines`] / (summary[`rspackOnly${type}Lines`] + summary[`webpackOnly${type}Lines`] + summary[`common${type}Lines`]))}
Lines(Common Modules),${summary[`rspackOnly${type}LinesInCommonModules`]},${summary[`common${type}LinesInCommonModules`]},${summary[`webpackOnly${type}LinesInCommonModules`]},${toPercent(summary[`common${type}LinesInCommonModules`] / (summary[`rspackOnly${type}LinesInCommonModules`] + summary[`webpackOnly${type}LinesInCommonModules`] + summary[`common${type}LinesInCommonModules`]))}`
        , ",", true);
    };

    if (summary.success) {
      output.push('\n');
      output.push(tablify(''));
      output.push('\n');
      output.push(tablify('Runtime'));
    } else {
      output.push('> Failed');
    }
    result.push(output.join('\n'));
  }
  fs.writeFileSync(path.join(__dirname, '../diff_output'), result.join('\n---\n'));
})();

