const styles = require('./entry').default
require('./entry2')

it("css hmr", (done) => {
	expect(styles).toHaveProperty('foo')
	NEXT(require("../../update")(done, true, () => {
		const updatedStyles = require('./entry').default
		expect(updatedStyles).toHaveProperty('bar')
		done()
	}));
});
