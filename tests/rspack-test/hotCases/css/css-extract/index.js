const styles = require('./entry').default
require('./entry2')

it("css hmr", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(styles).toHaveProperty('foo')
	NEXT(require("../../update")(done, true, () => {
		const updatedStyles = require('./entry').default
		expect(updatedStyles).toHaveProperty('bar')
		done()
	}));
}));
