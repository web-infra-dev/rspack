import { parse } from "@babel/parser";
import generate from "@babel/generator";
import traverse from "@babel/traverse";
import * as T from "@babel/types";
import { replaceModuleArgument } from "./replace-module-argument";

export interface IFormatCodeOptions {
	replacements?: Record<string, string>;
	ignorePropertyQuotationMark: boolean;
	ignoreModuleId: boolean;
	ignoreModuleArguments: boolean;
	ignoreBlockOnlyStatement: boolean;
	ignoreSwcHelpersPath: boolean;
	ignoreObjectPropertySequence: boolean;
	ignoreCssFilePath: boolean;
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
		IfStatement(path) {
			if (options.ignoreBlockOnlyStatement) {
				const consequent = path.get("consequent");
				if (
					consequent.isBlockStatement() &&
					consequent.node.body.length === 1
				) {
					consequent.replaceWith(consequent.node.body[0]);
				}
				const alternate = path.get("alternate");
				if (alternate.isBlockStatement() && alternate.node.body.length === 1) {
					alternate.replaceWith(alternate.node.body[0]);
				}
			}
		},
		For(path) {
			if (options.ignoreBlockOnlyStatement) {
				const body = path.get("body");
				if (body.isBlockStatement() && body.node.body.length === 1) {
					body.replaceWith(body.node.body[0]);
				}
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
		concise: false
	}).code;

	if (options.ignoreModuleArguments) {
		result = replaceModuleArgument(result);
	}

	if (options.replacements) {
		for (let [key, value] of Object.entries(options.replacements)) {
			result = result.split(key).join(value);
		}
	}

	return result.trim();
}
