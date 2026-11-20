export default function (source) {
	const { name, emitResult } = this.getOptions();
	const entries =
		source && source.trim().startsWith('[') ? JSON.parse(source) : [];
	entries.push({
		phase: 'normal',
		name,
		parallel: Boolean(this.parallel),
		pitchData: this.data?.pitch ?? null,
	});
	if (emitResult) {
		return `module.exports = ${JSON.stringify(entries)};`;
	}
	return JSON.stringify(entries);
}
