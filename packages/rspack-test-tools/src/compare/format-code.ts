import generate from "@babel/generator";
import { parse } from "@babel/parser";
import traverse, { NodePath } from "@babel/traverse";
import * as T from "@babel/types";

import { replaceModuleArgument } from "./replace-module-argument";

export interface IFormatCodeOptions {
	replacements?: IFormatCodeReplacement[];
	ignorePropertyQuotationMark: boolean;
	ignoreModuleId: boolean;
	ignoreModuleArguments: boolean;
	ignoreBlockOnlyStatement: boolean;
	ignoreSwcHelpersPath: boolean;
	ignoreObjectPropertySequence: boolean;
	ignoreCssFilePath: boolean;
	ignoreIfCertainCondition: boolean;
}

export interface IFormatCodeReplacement {
	from: string | RegExp;
	to: string;
}

const SWC_HELPER_PATH_REG =
	/^_swc_helpers_[a-zA-Z\d_-]+__WEBPACK_IMPORTED_MODULE_xxx__$/;
const CSS_FILE_EXT_REG = /(le|sa|c|sc)ss$/;
const INVALID_PATH_REG = /[<>:"/\\|?*.]/g;

export function formatCode(
	name: string,
	raw: string,
	options: IFormatCodeOptions
) {
	const ast = parse(raw, {
		sourceType: "unambiguous"
	});
	traverse(ast, {
		BlockStatement(path) {
			if (options.ignoreBlockOnlyStatement) {
				if (path.parentPath.isBlockStatement()) {
					path.replaceWithMultiple(path.node.body);
				}
			}
		},
		ObjectProperty(path) {
			if (options.ignorePropertyQuotationMark) {
				const keyPath = path.get("key");
				if (keyPath.isIdentifier()) {
					keyPath.replaceWith(T.stringLiteral(keyPath.node.name));
				}
			}
			if (options.ignoreCssFilePath && CSS_FILE_EXT_REG.test(name)) {
				const valuePath = path.get("value");
				if (valuePath.isStringLiteral()) {
					valuePath.node.value = valuePath.node.value.replace(
						INVALID_PATH_REG,
						"-"
					);
				}
			}
		},
		Identifier(path) {
			if (options.ignoreModuleId) {
				path.node.name = path.node.name.replace(
					/__WEBPACK_IMPORTED_MODULE_\d+__/g,
					"__WEBPACK_IMPORTED_MODULE_xxx__"
				);
			}
			if (options.ignoreSwcHelpersPath) {
				if (SWC_HELPER_PATH_REG.test(path.node.name)) {
					path.node.name = `$$SWC_HELPERS$$`;
				}
			}
		},
		StringLiteral(path) {
			if (path.node.extra?.raw) {
				path.node.extra.raw = JSON.stringify(path.node.extra.rawValue);
			}
		},
		IfStatement(path) {
			let consequentNode;
			let alternateNode;
			if (options.ignoreBlockOnlyStatement) {
				const consequent = path.get("consequent");
				if (
					consequent.isBlockStatement() &&
					consequent.node.body.length === 1
				) {
					consequent.node.body[0].leadingComments =
						consequent.node.leadingComments;
					consequentNode = consequent.node.body[0];
					consequent.replaceWith(consequentNode);
				}
				const alternate = path.get("alternate");
				if (alternate.isBlockStatement() && alternate.node.body.length === 1) {
					alternate.node.body[0].leadingComments =
						alternate.node.leadingComments;

					alternateNode = alternate.node.body[0];
					alternate.replaceWith(alternateNode);
				}
			}
			if (options.ignoreIfCertainCondition) {
				const testExpr = path.get("test");
				const testResult = testExpr.isBooleanLiteral()
					? testExpr.node.value
					: undefined;
				if (typeof testResult === "boolean") {
					if (testResult) {
						const consequent = path.get("consequent");
						if (consequent.isBlockStatement()) {
							if (consequentNode) {
								path.replaceWith(consequentNode);
							} else {
								path.replaceWithMultiple(consequent.node.body);
							}
						} else {
							path.replaceWith(consequent);
						}
					} else {
						const alternate = path.get("alternate");
						if (alternate.isBlockStatement()) {
							if (alternateNode) {
								path.replaceWith(alternateNode);
							} else {
								path.replaceWithMultiple(alternate.node.body);
							}
						} else if (alternate.isStatement()) {
							path.replaceWith(alternate);
						} else {
							path.remove();
						}
					}
				}
			}
		},
		For(path) {
			if (options.ignoreBlockOnlyStatement) {
				const body = path.get("body");
				if (body.isBlockStatement() && body.node.body.length === 1) {
					body.node.body[0].leadingComments = body.node.leadingComments;
					body.replaceWith(T.cloneNode(body.node.body[0], true, false));
				}
			}
		},
		While(path) {
			if (options.ignoreBlockOnlyStatement) {
				const body = path.get("body");
				if (body.isBlockStatement() && body.node.body.length === 1) {
					body.node.body[0].leadingComments = body.node.leadingComments;
					body.replaceWith(T.cloneNode(body.node.body[0], true, false));
				}
			}
		},
		SwitchCase(path) {
			if (
				path.node.consequent.length === 1 &&
				path.node.consequent[0].type === "BlockStatement"
			) {
				path.node.consequent = path.node.consequent[0].body;
			}
		},
		ObjectExpression(path) {
			if (options.ignoreObjectPropertySequence) {
				let result = [];
				let safe = [];
				while (path.node.properties.length || safe.length) {
					const cur = path.node.properties.shift()!;
					if (cur && T.isObjectProperty(cur)) {
						if (T.isIdentifier(cur.key)) {
							safe.push({
								name: cur.key.name,
								node: cur
							});
							continue;
						}
						if (T.isStringLiteral(cur.key)) {
							safe.push({
								name: cur.key.value,
								node: cur
							});
							continue;
						}
					}
					if (safe.length) {
						safe.sort((a, b) => (a.name > b.name ? 1 : -1));
						result.push(...safe.map(n => n.node));
						safe = [];
					}
					if (cur) {
						result.push(cur);
					}
				}
				path.node.properties = result;
			}
		}
	});
	let result = generate(ast, {
		comments: false,
		compact: false,
		concise: false,
		jsescOption: {
			quotes: "double"
		}
	}).code;

	if (options.ignoreModuleArguments) {
		result = replaceModuleArgument(result);
	}

	if (options.replacements) {
		for (let { from, to } of options.replacements) {
			result = result.replaceAll(from, to);
		}
	}

	// result of generate() is not stable with comments sometimes
	// so do it again
	return generate(
		parse(result, {
			sourceType: "unambiguous"
		}),
		{
			comments: false,
			compact: false,
			concise: false,
			jsescOption: {
				quotes: "double"
			}
		}
	).code.trim();
}
