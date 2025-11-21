import * as styles from "./index.module.css";

it("should generate correct exports", () => {
	const fs = __non_webpack_require__('fs')
	const path = __non_webpack_require__('path')
	expect(styles).toEqual(
		nsObj({
			a: '"aaa" 123',
			b: "multiple lines  bbb",
			'a/b': 'a/b-./'
		})
	);

	const css = fs.readFileSync(path.resolve(__dirname, './bundle0.css')).toString()
	const escape = css.replaceAll('\\', '')
	expect(escape).toContain(styles['a/b'])
});
