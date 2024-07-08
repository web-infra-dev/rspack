/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/node/nodeConsole.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import * as util from "util";
import { truncateArgs } from "../logging/truncateArgs";

// @ts-expect-error
export = ({ colors, appendOnly, stream }) => {
	// @ts-expect-error
	let currentStatusMessage = undefined;
	let hasStatusMessage = false;
	let currentIndent = "";
	let currentCollapsed = 0;
	// @ts-expect-error
	const indent = (str, prefix, colorPrefix, colorSuffix) => {
		if (str === "") return str;
		prefix = currentIndent + prefix;
		if (colors) {
			return (
				prefix +
				colorPrefix +
				str.replace(/\n/g, colorSuffix + "\n" + prefix + colorPrefix) +
				colorSuffix
			);
		} else {
			return prefix + str.replace(/\n/g, "\n" + prefix);
		}
	};

	const clearStatusMessage = () => {
		if (hasStatusMessage) {
			stream.write("\x1b[2K\r");
			hasStatusMessage = false;
		}
	};

	const writeStatusMessage = () => {
		// @ts-expect-error
		if (!currentStatusMessage) return;
		const l = stream.columns;
		const args = l
			? truncateArgs(currentStatusMessage, l - 1)
			: currentStatusMessage;
		const str = args.join(" ");
		const coloredStr = `\u001b[1m${str}\u001b[39m\u001b[22m`;
		stream.write(`\x1b[2K\r${coloredStr}`);
		hasStatusMessage = true;
	};
	// @ts-expect-error
	const writeColored = (prefix, colorPrefix, colorSuffix) => {
		// @ts-expect-error
		return (...args) => {
			if (currentCollapsed > 0) return;
			clearStatusMessage();
			const str = indent(
				util.format(...args),
				prefix,
				colorPrefix,
				colorSuffix
			);
			stream.write(str + "\n");
			writeStatusMessage();
		};
	};

	const writeGroupMessage = writeColored(
		"<-> ",
		"\u001b[1m\u001b[36m",
		"\u001b[39m\u001b[22m"
	);

	const writeGroupCollapsedMessage = writeColored(
		"<+> ",
		"\u001b[1m\u001b[36m",
		"\u001b[39m\u001b[22m"
	);

	return {
		log: writeColored("    ", "\u001b[1m", "\u001b[22m"),
		debug: writeColored("    ", "", ""),
		trace: writeColored("    ", "", ""),
		info: writeColored("<i> ", "\u001b[1m\u001b[32m", "\u001b[39m\u001b[22m"),
		warn: writeColored("<w> ", "\u001b[1m\u001b[33m", "\u001b[39m\u001b[22m"),
		error: writeColored("<e> ", "\u001b[1m\u001b[31m", "\u001b[39m\u001b[22m"),
		logTime: writeColored(
			"<t> ",
			"\u001b[1m\u001b[35m",
			"\u001b[39m\u001b[22m"
		),
		// @ts-expect-error
		group: (...args) => {
			writeGroupMessage(...args);
			if (currentCollapsed > 0) {
				currentCollapsed++;
			} else {
				currentIndent += "  ";
			}
		},
		// @ts-expect-error
		groupCollapsed: (...args) => {
			writeGroupCollapsedMessage(...args);
			currentCollapsed++;
		},
		groupEnd: () => {
			if (currentCollapsed > 0) currentCollapsed--;
			else if (currentIndent.length >= 2)
				currentIndent = currentIndent.slice(0, currentIndent.length - 2);
		},

		// @ts-expect-error
		profile: console.profile && (name => console.profile(name)),

		// @ts-expect-error
		profileEnd: console.profileEnd && (name => console.profileEnd(name)),
		clear:
			!appendOnly &&
			console.clear &&
			(() => {
				clearStatusMessage();

				console.clear();
				writeStatusMessage();
			}),
		status: appendOnly
			? writeColored("<s> ", "", "")
			: // @ts-expect-error
				(name, ...args) => {
					args = args.filter(Boolean);
					if (name === undefined && args.length === 0) {
						clearStatusMessage();
						currentStatusMessage = undefined;
					} else if (
						typeof name === "string" &&
						name.startsWith("[webpack.Progress] ")
					) {
						currentStatusMessage = [name.slice(19), ...args];
						writeStatusMessage();
					} else if (name === "[webpack.Progress]") {
						currentStatusMessage = [...args];
						writeStatusMessage();
					} else {
						currentStatusMessage = [name, ...args];
						writeStatusMessage();
					}
				}
	};
};
