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
