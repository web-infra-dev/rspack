// @ts-nocheck

const { SummaryReporter } = require("@jest/reporters");
const chalk =
	require.cache[require.resolve("@jest/reporters")].require("chalk");

const isUpdate =
	process.argv.includes("-u") || process.argv.includes("--updateSnapshot");
const isFiltering =
	process.argv.includes("-t") || process.argv.includes("--testNamePattern");
const isVerbose = process.argv.includes("--verbose");

if (!isVerbose && !isUpdate && isFiltering) {
	class IgnoreSnapshotSummaryReporter extends SummaryReporter {
		_printSnapshotSummary(snapshots, globalConfig) {
			if (
				snapshots.added ||
				snapshots.filesRemoved ||
				snapshots.unchecked ||
				snapshots.unmatched ||
				snapshots.updated
			) {
				this.log(
					chalk.bold.yellow(
						"Some snapshots are obsoleted, flush with `npm run test -- -u` if necessary.\n"
					)
				);
			}
		}
	}
	module.exports = IgnoreSnapshotSummaryReporter;
} else {
	module.exports = SummaryReporter;
}
