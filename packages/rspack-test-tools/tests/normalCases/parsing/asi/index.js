import { foo } from "./a"
var a = {
	a: 1
}
foo()
const b = (function() { throw new Error("do not callme") })
foo()
