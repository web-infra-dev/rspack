import * as styles from './index.module.css'

it("should transform css correct", () => {
	const fs = require('fs')
	const path = require('path')

	expect(styles).toHaveProperty('used');
	expect('unused' in styles).toBeFalsy();

	expect(fs.readFileSync(path.resolve(__dirname, './bundle0.css')).toString()).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'bundle0.css.txt'))
});
