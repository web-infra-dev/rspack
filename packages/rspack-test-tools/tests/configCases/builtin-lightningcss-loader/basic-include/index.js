import * as styles from './index.module.css'

it("should transform css correct", () => {
	const fs = __non_webpack_require__('fs')
	const path = __non_webpack_require__('path')

	expect(styles).toHaveProperty('used');
	expect('unused' in styles).toBeFalsy();

	expect(fs.readFileSync(path.resolve(__dirname, './bundle0.css')).toString()).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'bundle0.css.txt'))
});
