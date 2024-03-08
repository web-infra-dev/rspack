// @ts-nocheck

const PATH_QUERY_FRAGMENT_REGEXP =
	/^((?:\0.|[^?#\0])*)(\?(?:\0.|[^#\0])*)?(#.*)?$/;

export const parseResource = str => {
	const match = PATH_QUERY_FRAGMENT_REGEXP.exec(str);
	return {
		resource: str,
		path: match[1].replace(/\0(.)/g, "$1"),
		query: match[2] ? match[2].replace(/\0(.)/g, "$1") : "",
		fragment: match[3] || ""
	};
};
