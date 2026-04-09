module.exports = {
	findBundle() {
		return ['index.mjs']
	},
	snapshotFileFilter(file) {
		return (
			(file.endsWith('.mjs') || file.endsWith('.js') || file.endsWith('.css')) &&
			!file.includes('runtime.js')
		);
	},
}
