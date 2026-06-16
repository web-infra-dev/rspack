const fs = require("fs")
export const foo1 = () => {}
export const foo2 = () => {}
const bar = "bar";
export default bar

it("should success compile and work",()=>{
	const output = fs.readFileSync(__filename).toString();
	expect(output.match(/exports(\[|\.)/g).length).toBe(
		globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK ? 8 : 4
	)
})
