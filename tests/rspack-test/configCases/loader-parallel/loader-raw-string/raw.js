function loader(content, sourceMap, meta) {
	const { ensureObject } = require("./loader-util");
	meta = ensureObject(meta);
	(meta.data = meta.data || []).push(Buffer.isBuffer(content));

	if (meta.data.length === 3) {
		return `module.exports = ${JSON.stringify(meta.data)};`;
	}

	this.callback(null, content, null, meta);
}
loader.displayName = "raw-loader";
loader.raw = true;

module.exports = loader;
