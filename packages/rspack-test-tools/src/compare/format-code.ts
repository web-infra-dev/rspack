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
}

const SWC_HELPER_PATH_REG =
	/^_swc_helpers_[a-zA-Z\d_-]+__WEBPACK_IMPORTED_MODULE_xxx__$/;

export function formatCode(raw: string, options: IFormatCodeOptions) {
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
