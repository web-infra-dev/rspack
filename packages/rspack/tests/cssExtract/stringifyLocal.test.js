import { stringifyLocal } from "../../src/builtin-plugin/mini-css-extract/utils";

describe("stringifyLocal", () => {
	it(`primitive`, async () => {
		const testObj = "classA";

		expect(stringifyLocal(testObj)).toBe('"classA"');
	});

	it(`arrow function`, async () => {
		const testFn = () => "classA";

		expect(stringifyLocal(testFn)).toBe('()=>"classA"');
	});

	it(`function`, async () => {
		const testFn = function testFn() {
			return "classA";
		};

		expect(stringifyLocal(testFn)).toMatchInlineSnapshot(`
		"function testFn() {
		            return "classA";
		        }"
	`);
	});
});
