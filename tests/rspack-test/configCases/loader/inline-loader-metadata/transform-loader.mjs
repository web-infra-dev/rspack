export default function (source) {
	const { value } = this.getOptions();
	return `module.exports = ${JSON.stringify({
		value,
		parallel: Boolean(this.parallel),
	})};`;
}
