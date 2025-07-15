
// https://github.com/web-infra-dev/rspack/issues/10889
it('should parse invalid regex syntax', () => {
	const a = ''.replace(/[-/\\^$*+?.()|[\]{}]/g, '');
	const b = ''.replace(/a/i, '');
	const c = ''.replace(/a/m, '');
	const d = ''.replace(/a/s, '');
	const f = ''.replace(/a/y, '');
	const h = ''.replace(/abc/, '');

	console.log('regex:', a, b, c, d, f, h)
})

