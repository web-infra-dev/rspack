import { ensureObject } from "./loader-util.mjs";

function loader(content, sourceMap, meta) {

	meta = ensureObject(meta);
	(meta.data = meta.data || []).push(Buffer.isBuffer(content));

	if (meta.data.length === 3) {
		return `module.exports = ${JSON.stringify(meta.data)};`;
	}

	this.callback(null, content, null, meta);
}
loader.displayName = "string-loader";

export default loader;
