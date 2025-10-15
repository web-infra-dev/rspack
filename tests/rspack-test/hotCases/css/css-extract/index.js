const styles = require('./entry').default
require('./entry2')

it("css hmr", (done) => {
	expect(styles).toHaveProperty('foo')
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
		const updatedStyles = require('./entry').default
		expect(updatedStyles).toHaveProperty('bar')
		done()
	}));
});
