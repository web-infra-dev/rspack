export default function (content, _sourceMap, additionalData) {
	return `module.exports = ${JSON.stringify({
		parallel: Boolean(this.parallel),
		content,
		additionalData,
	})};`;
}
