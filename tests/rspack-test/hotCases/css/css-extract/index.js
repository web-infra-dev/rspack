const styles = require('./entry').default
require('./entry2')

it("css hmr", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(styles).toHaveProperty('foo')
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
		const updatedStyles = require('./entry').default
		expect(updatedStyles).toHaveProperty('bar')
		done()
	}));
}));
