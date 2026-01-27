const path = require('path');
const fs = require('fs');

it ('some expressions should not be handled by APIPlugin', () => {
	const file = path.resolve(__dirname, 'bundle0.js')
	const content = fs.readFileSync(file, 'utf-8');
	expect(content).toContain(`
console.log(require.cache)
console.log(require.extensions)
console.log(require.config)
console.log(require.version)
console.log(require.include)
console.log(require.onError)
console.log(typeof module)
`)
})

it ('`typeof module` should be intercepted by Rslib Plugin', () => {
	const file = path.resolve(__dirname, 'bundle1.mjs')
	const content = fs.readFileSync(file, 'utf-8');
	expect(content).toBe(`import node_module from "node:module";

;// CONCATENATED MODULE: external "node:module"

;// CONCATENATED MODULE: ./module.js


console.log(typeof node_module)

`)
})
