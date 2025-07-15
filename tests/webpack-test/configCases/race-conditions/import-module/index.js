import * as styles1 from './module.js';
import * as styles from './style.module.css';

it("should not deadlock when using importModule", () => {
	expect(styles.someBottom).toBe("8px");
	expect(styles1.someBottom).toBe("8px");
});
