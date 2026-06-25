import './change'

const fs = require('fs')
const path = require('path')

it("should have correct order", async () => {
	let content = fs.readFileSync(path.resolve(__dirname, './bundle.css')).toString()
	expect(content.replaceAll('\n', '').trim()).toBe('.a{}.b{}')
	await NEXT_HMR();
	content = fs.readFileSync(path.resolve(__dirname, './bundle.css')).toString()
	expect(content.replaceAll('\n', '').trim()).toBe('.b{}.a{}')
});

module.hot.accept("./change");

