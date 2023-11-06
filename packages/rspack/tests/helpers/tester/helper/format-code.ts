import { parse } from "@babel/parser";
import generate from "@babel/generator";
import traverse from "@babel/traverse";
import * as T from "@babel/types";
import { replaceModuleArgument } from "./replace-module-argument";

export interface IFormatCodeOptions {
	replacements?: Record<string, string>;
	ignorePropertyQuotationMark: boolean;
	ignoreModuleId: boolean;
	ignoreModuleArugments: boolean;
}

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
		}
	});
	let result = generate(ast, {
		comments: false,
		compact: false,
		concise: false
	}).code;

	if (options.ignoreModuleArugments) {
		result = replaceModuleArgument(result);
	}

	if (options.replacements) {
		for (let [key, value] of Object.entries(options.replacements)) {
			result = result.split(key).join(value);
		}
	}

	return result.trim();
}
