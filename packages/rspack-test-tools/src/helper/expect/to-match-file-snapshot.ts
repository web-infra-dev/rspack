// @ts-nocheck

/* istanbul ignore file */

import fs from "fs";
import path from "path";
import chalk from "chalk";
import filenamify from "filenamify";
import { diff } from "jest-diff";
import mkdirp from "mkdirp";

/**
 * Check if 2 strings or buffer are equal
 * @param {string | Buffer} a
 * @param {string | Buffer} b
 */
const isEqual = (a, b) => {
	// @ts-ignore: TypeScript gives error if we pass string to buffer.equals
	return Buffer.isBuffer(a) ? a.equals(b) : a === b;
};

/**
 * Match given content against content of the specified file.
 *
 * @param {string | Buffer} content Output content to match
 * @param {string} [filepath] Path to the file to match against
 * @param {{ diff?: import('jest-diff').DiffOptions }} options Additional options for matching
 * @this {{ testPath: string, currentTestName: string, assertionCalls: number, isNot: boolean, snapshotState: { added: number, updated: number, unmatched: number, _updateSnapshot: 'none' | 'new' | 'all' } }}
 */
export function toMatchFileSnapshot(content, filepath, options = {}) {
	const { isNot, snapshotState } = this;

	const filename =
		filepath === undefined
			? // If file name is not specified, generate one from the test title
				path.join(
					path.dirname(this.testPath),
					"__file_snapshots__",
					`${filenamify(this.currentTestName, {
						replacement: "-"
					}).replace(/\s/g, "-")}-${this.assertionCalls}`
				)
			: filepath;

	options = {
		// Options for jest-diff
		diff: Object.assign(
			{
				expand: false,
				contextLines: 5,
				aAnnotation: "Snapshot"
			},
			options.diff || {}
		)
	};

	if (snapshotState._updateSnapshot === "none" && !fs.existsSync(filename)) {
		// We're probably running in CI environment

		snapshotState.unmatched++;

		return {
			pass: isNot,
			message: () =>
				`New output file ${chalk.blue(
					path.basename(filename)
				)} was ${chalk.bold.red("not written")}.\n\n` +
				"The update flag must be explicitly passed to write a new snapshot.\n\n" +
				`This is likely because this test is run in a ${chalk.blue(
					"continuous integration (CI) environment"
				)} in which snapshots are not written by default.\n\n`
		};
	}

	if (fs.existsSync(filename)) {
		const output = fs
			.readFileSync(filename, Buffer.isBuffer(content) ? null : "utf8")
			.replace(/\r\n/g, "\n");

		if (isNot) {
			// The matcher is being used with `.not`

			if (!isEqual(content, output)) {
				// The value of `pass` is reversed when used with `.not`
				return { pass: false, message: () => "" };
			} else {
				snapshotState.unmatched++;

				return {
					pass: true,
					message: () =>
						`Expected received content ${chalk.red(
							"to not match"
						)} the file ${chalk.blue(path.basename(filename))}.`
				};
			}
		} else {
			if (isEqual(content, output)) {
				return { pass: true, message: () => "" };
			} else {
				if (snapshotState._updateSnapshot === "all") {
					mkdirp.sync(path.dirname(filename));
					fs.writeFileSync(filename, content);

					snapshotState.updated++;

					return { pass: true, message: () => "" };
				} else {
					snapshotState.unmatched++;

					const difference =
						Buffer.isBuffer(content) || Buffer.isBuffer(output)
							? ""
							: `\n\n${diff(output, content, options.diff)}`;

					return {
						pass: false,
						message: () =>
							`Received content ${chalk.red(
								"doesn't match"
							)} the file ${chalk.blue(path.basename(filename))}.${difference}`
					};
				}
			}
		}
	} else {
		if (
			!isNot &&
			(snapshotState._updateSnapshot === "new" ||
				snapshotState._updateSnapshot === "all")
		) {
			mkdirp.sync(path.dirname(filename));
			fs.writeFileSync(filename, content);

			snapshotState.added++;

			return { pass: true, message: () => "" };
		} else {
			snapshotState.unmatched++;

			return {
				pass: true,
				message: () =>
					`The output file ${chalk.blue(
						path.basename(filename)
					)} ${chalk.bold.red("doesn't exist")}.`
			};
		}
	}
}
