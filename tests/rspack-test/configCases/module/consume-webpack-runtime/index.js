import { __webpack_require__ as namedUse } from './runtime-export-named'
import defaultUse from './runtime-export-default'

it("should compile and run", () => {
	expect(namedUse()).toBe(42);
	expect(defaultUse()).toBe(42);

	const path = __non_webpack_require__('path')
	const fs = __non_webpack_require__('fs')
	{
		const content = fs.readFileSync(path.resolve(__dirname, './bundle0.js'), 'utf-8')
		const NESTED_RE = /__nested_webpack_require_(.+)__/
		expect(content.match(NESTED_RE)[1].length).toBeGreaterThan(0)
	}

	{
		const content = fs.readFileSync(path.resolve(__dirname, './bundle1.js'), 'utf-8')
		const NESTED_RE = /__nested_webpack_require_(.+)__/
		expect(content.match(NESTED_RE)[1].length).toBeGreaterThan(0)
	}
});
