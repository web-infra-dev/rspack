/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
	  parser: {
		javascript: {
		  wrappedContextRegExp: /.*1/,
		}
	  }
	},
  }
  