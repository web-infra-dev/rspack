// @ts-nocheck

const PATH_QUERY_FRAGMENT_REGEXP =
	/^((?:\u200b.|[^?#\u200b])*)(\?(?:\u200b.|[^#\u200b])*)?(#.*)?$/;

export const parseResource = str => {
	const match = PATH_QUERY_FRAGMENT_REGEXP.exec(str);
	return {
		resource: str,
		path: match[1].replace(/\u200b(.)/g, "$1"),
		query: match[2] ? match[2].replace(/\u200b(.)/g, "$1") : "",
		fragment: match[3] || ""
	};
};
