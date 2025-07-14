
// https://github.com/web-infra-dev/rspack/issues/10889
describe('should parse invalid regex syntax', () => {
	const foo = '`~!@#$%^&*()-=+[{]}\\|;:\'",.<>/?'.replace(
		/[-/\\^$*+?.()|[\]{}]/g,
		'\\$&'
	);
	console.log('foo')
})

