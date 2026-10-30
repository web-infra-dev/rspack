import { parseSync } from "@swc-next/parser";

/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	const { program: ast, comments, diagnostics } = parseSync(source, {
		sourceType: "module",
		preserveParens: false,
		comments: "flat",
		semanticErrors: true
	});
	const errors = diagnostics.filter(diagnostic => diagnostic.severity === "error");
	if (errors.length > 0) {
		this.callback(new Error(errors.map(error => error.message).join("\n")));
		return;
	}

	// change something to test if it's really used
	//@ts-ignore
	ast.body[0].expression.right.arguments[0].value = "./ok";
	//@ts-ignore
	ast.body[0].expression.right.arguments[0].raw = '"./ok"';

	//@ts-ignore
	ast.comments = comments;
	this.callback(null, source, undefined, {
		webpackAST: ast
	});
};
