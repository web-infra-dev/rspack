const { SummaryReporter } = require("@jest/reporters");
const chalk = require.cache[require.resolve('@jest/reporters')].require("chalk");

if (!process.argv.includes("--verbose") && (process.argv.includes("-t") || process.argv.some(i => i.includes("--testNamePattern")))) {
  class IgnoreSnapshotSummaryReporter extends SummaryReporter {
    _printSnapshotSummary(snapshots, globalConfig) {
      if (
        snapshots.added ||
        snapshots.filesRemoved ||
        snapshots.unchecked ||
        snapshots.unmatched ||
        snapshots.updated
      ) {
        this.log(chalk.bold.yellow('Some snapshots are obsoleted, flush with `npm run test -- -u` if necessary.\n'));
      }
    }
  }
  module.exports = IgnoreSnapshotSummaryReporter;
} else {
  module.exports = SummaryReporter;
}