module.exports = {
	findBundle() {
		return ['index.mjs']
	},
	snapshotFileFilter(file) {
		return (
			(file.endsWith('.mjs') || file.endsWith('.js') || file.endsWith('.png')) &&
			!file.includes('runtime.js')
		);
	},
	// Replace the PNG binary content with a placeholder so the snapshot only
	// asserts that the file was emitted at the preserved path.
	snapshotContent(content) {
		return content.replace(/(```png title=[^\n]+\n)[\s\S]*?(\n```)/g, '$1<binary>$2');
	},
}
