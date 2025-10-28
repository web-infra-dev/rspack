// @ts-nocheck

const { DefaultReporter } = require("@jest/reporters");
const chalk =
	require.cache[require.resolve("@jest/reporters")].require("chalk");
const jestUtil =
	require.cache[require.resolve("@jest/reporters")].require("jest-util");

const ARROW = " \u203A ";
const DOT = " \u2022 ";
const FAIL_COLOR = chalk.bold.red;
const SNAPSHOT_ADDED = chalk.bold.green;
const SNAPSHOT_UPDATED = chalk.bold.green;
const SNAPSHOT_OUTDATED = chalk.bold.yellow;
function getSnapshotStatus(snapshot, afterUpdate) {
	const statuses = [];
	if (snapshot.added) {
		statuses.push(
			SNAPSHOT_ADDED(
				`${ARROW + jestUtil.pluralize("snapshot", snapshot.added)} written.`
			)
		);
	}
	if (snapshot.updated) {
		statuses.push(
			SNAPSHOT_UPDATED(
				`${ARROW + jestUtil.pluralize("snapshot", snapshot.updated)} updated.`
			)
		);
	}
	if (snapshot.unmatched) {
		statuses.push(
			FAIL_COLOR(
				`${ARROW + jestUtil.pluralize("snapshot", snapshot.unmatched)} failed.`
			)
		);
	}
	if (snapshot.unchecked) {
		if (afterUpdate) {
			statuses.push(
				SNAPSHOT_UPDATED(
					`${
						ARROW + jestUtil.pluralize("snapshot", snapshot.unchecked)
					} removed.`
				)
			);
			for (const key of snapshot.uncheckedKeys) {
				statuses.push(`  ${DOT}${key}`);
			}
		} else {
			statuses.push(
				`${SNAPSHOT_OUTDATED(
					`${
						ARROW + jestUtil.pluralize("snapshot", snapshot.unchecked)
					} obsolete`
				)}.`
			);
		}
	}
	if (snapshot.fileDeleted) {
		statuses.push(SNAPSHOT_UPDATED(`${ARROW}snapshot file removed.`));
	}
	return statuses;
}

const isUpdate =
	process.argv.includes("-u") || process.argv.includes("--updateSnapshot");
const isFiltering =
	process.argv.includes("-t") || process.argv.includes("--testNamePattern");
const isVerbose = process.argv.includes("--verbose");

if (!isVerbose && !isUpdate && isFiltering) {
	class IgnoreSnapshotDefaultReporter extends DefaultReporter {
		printTestFileFailureMessage(_testPath, _config, result) {
			if (result.failureMessage) {
				this.log(result.failureMessage);
			}
			const didUpdate = this._globalConfig.updateSnapshot === "all";
			const snapshotStatuses = getSnapshotStatus(result.snapshot, didUpdate);
			snapshotStatuses.forEach(this.log);
		}
	}
	module.exports = IgnoreSnapshotDefaultReporter;
} else {
	module.exports = DefaultReporter;
}
