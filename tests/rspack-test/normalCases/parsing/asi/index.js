import { foo } from "./a"
foo()

var a = {
	a: 1
}
foo()
const b = (function() { throw new Error("do not callme") })
foo()

export const fooA = foo
foo()

export { foo as fooB }
foo()

export default a
foo()

export * from "./a.js"
foo()

debugger
foo()

export function a() {}
function bb() {
  a(), foo();
}

function d() {}
export function c() {}
d(), foo();

const tpl = `${1}tpl`
const arr = []
foo(arr, tpl)
