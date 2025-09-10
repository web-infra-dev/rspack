import * as styles from './index.module.css'

it("should transform css correct", () => {
	expect(styles).toHaveProperty('used');
	expect('unused' in styles).toBeFalsy();
});
