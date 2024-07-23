/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/formatLocation.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

// Waiting to adapt
type DependencyLocation = any;
type SourcePosition = any;

/**
 * @param pos position
 * @returns formatted position
 */
const formatPosition = (pos: SourcePosition): string => {
	if (pos && typeof pos === "object") {
		if ("line" in pos && "column" in pos) {
			return `${pos.line}:${pos.column}`;
		}
		if ("line" in pos) {
			return `${pos.line}:?`;
		}
	}
	return "";
};

/**
 * @param loc location
 * @returns formatted location
 */
const formatLocation = (loc: DependencyLocation): string => {
	if (loc && typeof loc === "object") {
		if ("start" in loc && loc.start && "end" in loc && loc.end) {
			if (
				typeof loc.start === "object" &&
				typeof loc.start.line === "number" &&
				typeof loc.end === "object" &&
				typeof loc.end.line === "number" &&
				typeof loc.end.column === "number" &&
				loc.start.line === loc.end.line
			) {
				return `${formatPosition(loc.start)}-${loc.end.column}`;
			}
			if (
				typeof loc.start === "object" &&
				typeof loc.start.line === "number" &&
				typeof loc.start.column !== "number" &&
				typeof loc.end === "object" &&
				typeof loc.end.line === "number" &&
				typeof loc.end.column !== "number"
			) {
				return `${loc.start.line}-${loc.end.line}`;
			}
			return `${formatPosition(loc.start)}-${formatPosition(loc.end)}`;
		}
		if ("start" in loc && loc.start) {
			return formatPosition(loc.start);
		}
		if ("name" in loc && "index" in loc) {
			return `${loc.name}[${loc.index}]`;
		}
		if ("name" in loc) {
			return loc.name;
		}
	}
	return "";
};

export default formatLocation;
