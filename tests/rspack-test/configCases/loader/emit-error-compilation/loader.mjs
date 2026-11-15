export default function (content) {
	// This compilation-local side effect requires the loader to run on every build.
	this.cacheable(false)
	let error = new Error("error2")
	error.file = "./index.js:3:1"
	error.module = this._module
	this._compilation.errors.push(error)
	return content
}
