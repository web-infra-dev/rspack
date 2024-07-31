/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/Dependency.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

// https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Dependency.js#L30
export type SourcePosition = {
	line: number;
	column?: number;
};

// https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Dependency.js#L37
export type RealDependencyLocation = {
	start: SourcePosition;
	end?: SourcePosition;
	index?: number;
};

// https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Dependency.js#L44
export type SyntheticDependencyLocation = {
	name: string;
	index?: number;
};

// https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Dependency.js#L49
export type DependencyLocation =
	| SyntheticDependencyLocation
	| RealDependencyLocation;
