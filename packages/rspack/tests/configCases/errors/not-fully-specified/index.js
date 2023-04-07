import yes from './fully-specified.mjs'

it("should error when not fullySpecified for mjs", () => {
	expect(yes).toBe('fully-specified');
	try {
		require('./not-fully-specified')
	} catch (e) {
		expect(e.message.includes('Failed to resolve ./not-fully-specified')).toBe(true)
	}
});
