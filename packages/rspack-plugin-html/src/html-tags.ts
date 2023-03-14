/**
 * The following code is modified based on
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/lib/html-tags.js
 *
 * MIT Licensed
 * Author Jan Nicklas
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/LICENSE
 */
import { HtmlTagObject } from "./index";

/**
 * Turn a tag definition into a html string
 */
export function htmlTagObjectToString(
	tagDefinition: HtmlTagObject,
	xhtml: boolean
) {
	const attributes = Object.keys(tagDefinition.attributes || {})
		.filter(function (attributeName) {
			return (
				tagDefinition.attributes[attributeName] === "" ||
				tagDefinition.attributes[attributeName]
			);
		})
		.map(function (attributeName) {
			if (tagDefinition.attributes[attributeName] === true) {
				return xhtml
					? attributeName + '="' + attributeName + '"'
					: attributeName;
			}
			return (
				attributeName + '="' + tagDefinition.attributes[attributeName] + '"'
			);
		});
	return (
		"<" +
		[tagDefinition.tagName].concat(attributes).join(" ") +
		(tagDefinition.voidTag && xhtml ? "/" : "") +
		">" +
		(tagDefinition.innerHTML || "") +
		(tagDefinition.voidTag ? "" : "</" + tagDefinition.tagName + ">")
	);
}

/**
 * All html tag elements which must not contain innerHTML
 * @see https://www.w3.org/TR/html5/syntax.html#void-elements
 */
const voidTags = [
	"area",
	"base",
	"br",
	"col",
	"embed",
	"hr",
	"img",
	"input",
	"keygen",
	"link",
	"meta",
	"param",
	"source",
	"track",
	"wbr"
];

/**
 * Static helper to create a tag object to be get injected into the dom
 */
export function createHtmlTagObject(
	tagName: string,
	attributes: { [attributeName: string]: string | boolean | null | undefined },
	innerHTML: string,
	meta: { [attributeName: string]: string | boolean | null | undefined }
) {
	return {
		tagName: tagName,
		voidTag: voidTags.indexOf(tagName) !== -1,
		attributes: attributes || {},
		meta: meta || {},
		innerHTML: innerHTML
	};
}

/**
 * The `HtmlTagArray Array with a custom `.toString()` method.
 *
 * This allows the following:
 * ```
 *   const tags = HtmlTagArray.from([tag1, tag2]);
 *   const scriptTags = tags.filter((tag) => tag.tagName === 'script');
 *   const html = scriptTags.toString();
 * ```
 *
 * Or inside a string literal:
 * ```
 *   const tags = HtmlTagArray.from([tag1, tag2]);
 *   const html = `<html><body>${tags.filter((tag) => tag.tagName === 'script')}</body></html>`;
 * ```
 *
 */
export class HtmlTagArray extends Array {
	toString() {
		return this.join("");
	}
}
