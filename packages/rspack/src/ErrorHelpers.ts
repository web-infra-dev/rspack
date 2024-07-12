/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/ErrorHelpers.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const loaderFlag = "LOADER_EXECUTION";

const webpackOptionsFlag = "WEBPACK_OPTIONS";

export const cutOffByFlag = (stack: string, flag: string) => {
	const stacks = stack.split("\n");
	for (let i = 0; i < stacks.length; i++) {
		if (stacks[i].includes(flag)) {
			stacks.length = i;
		}
	}
	return stacks.join("\n");
};

export const cutOffLoaderExecution = (stack: string) =>
	cutOffByFlag(stack, loaderFlag);

export const cutOffWebpackOptions = (stack: string) =>
	cutOffByFlag(stack, webpackOptionsFlag);

export const cutOffMultilineMessage = (
	stack: string,
	message: string
): string => {
	const stacks = stack.split("\n");
	const messages = message.split("\n");

	const result: string[] = [];

	stacks.forEach((line, idx) => {
		if (!line.includes(messages[idx])) result.push(line);
	});

	return result.join("\n");
};

export const cutOffMessage = (stack: string, message: string) => {
	const nextLine = stack.indexOf("\n");
	if (nextLine === -1) {
		return stack === message ? "" : stack;
	} else {
		const firstLine = stack.slice(0, nextLine);
		return firstLine === message ? stack.slice(nextLine + 1) : stack;
	}
};

export const cleanUp = (stack: string, message: string): string => {
	let str = stack;
	str = cutOffLoaderExecution(stack);
	str = cutOffMessage(str, message);
	return str;
};

export const cleanUpWebpackOptions = (stack: string, message: string) => {
	let str = stack;
	str = cutOffWebpackOptions(str);
	str = cutOffMultilineMessage(str, message);
	return stack;
};
