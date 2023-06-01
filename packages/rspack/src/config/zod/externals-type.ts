import { z } from "zod";

export function externalsType() {
	return z.enum([
		"var",
		"module",
		"assign",
		"this",
		"window",
		"self",
		"global",
		"commonjs",
		"commonjs2",
		"commonjs-module",
		"commonjs-static",
		"amd",
		"amd-require",
		"umd",
		"umd2",
		"jsonp",
		"system",
		"promise",
		"import",
		"script",
		"node-commonjs"
	]);
}
