// @ts-nocheck

/* istanbul ignore file */

import fs from "node:fs";
import path from "node:path";
import chalk from "chalk";
import filenamify from "filenamify";
import { diff } from "jest-diff";
import type { FileMatcherOptions } from "../../../jest";

const { serialize } = require(
	path.join(path.dirname(require.resolve("jest-snapshot")), "./utils.js")
);
/**
 * Check if 2 strings or buffer are equal
 */
const isEqual = (a: string | Buffer, b: string | Buffer): boolean => {
	// @ts-ignore: TypeScript gives error if we pass string to buffer.equals
	return Buffer.isBuffer(a) ? a.equals(b) : a === b;
};

/**
 * Match given content against content of the specified file.
 *
 * @param content Output content to match
 * @param filepath Path to the file to match against
 * @param options Additional options for matching
 */
export function toMatchFileSnapshot(
	this: {
		testPath: string;
		currentTestName: string;
		assertionCalls: number;
		isNot: boolean;
		snapshotState: {
			added: number;
			updated: number;
			unmatched: number;
			_updateSnapshot: "none" | "new" | "all";
		};
	},
	rawContent: string | Buffer,
	filepath: string,
	options: FileMatcherOptions = {}
) {
	const content =
		Buffer.isBuffer(rawContent) || typeof rawContent === "string"
			? rawContent
			: serialize(rawContent);

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

	if (snapshotState._updateSnapshot === "none" && !fs.existsSync(filename)) {
		// We're probably running in CI environment

		snapshotState.unmatched++;

		return {
			pass: isNot,
			message: () =>
				`New output file ${chalk.blue(
					path.basename(filename)
				)} was ${chalk.bold.red("not written")}.\n\nThe update flag must be explicitly passed to write a new snapshot.\n\nThis is likely because this test is run in a ${chalk.blue(
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
			}
			snapshotState.unmatched++;

			return {
				pass: true,
				message: () =>
					`Expected received content ${chalk.red(
						"to not match"
					)} the file ${chalk.blue(path.basename(filename))}.`
			};
		}
		if (isEqual(content, output)) {
			return { pass: true, message: () => "" };
		}
		if (snapshotState._updateSnapshot === "all") {
			fs.mkdirSync(path.dirname(filename), { recursive: true });
			fs.writeFileSync(filename, content);

			snapshotState.updated++;

			return { pass: true, message: () => "" };
		}
		snapshotState.unmatched++;

		const difference =
			Buffer.isBuffer(content) || Buffer.isBuffer(output)
				? ""
				: `\n\n${diff(
						output,
						content,
						Object.assign(
							{
								expand: false,
								contextLines: 5,
								aAnnotation: "Snapshot"
							},
							options.diff || {}
						)
					)}`;

		return {
			pass: false,
			message: () =>
				`Received content ${chalk.red(
					"doesn't match"
				)} the file ${chalk.blue(path.basename(filename))}.${difference}`
		};
	}
	if (
		!isNot &&
		(snapshotState._updateSnapshot === "new" ||
			snapshotState._updateSnapshot === "all")
	) {
		fs.mkdirSync(path.dirname(filename), { recursive: true });
		fs.writeFileSync(filename, content);

		snapshotState.added++;

		return { pass: true, message: () => "" };
	}
	snapshotState.unmatched++;

	return {
		pass: true,
		message: () =>
			`The output file ${chalk.blue(
				path.basename(filename)
			)} ${chalk.bold.red("doesn't exist")}.`
	};
}
