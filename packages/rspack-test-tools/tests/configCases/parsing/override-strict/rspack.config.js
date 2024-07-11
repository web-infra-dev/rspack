/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		mode: "production",
		entry: ["./strict"],
		module: {
			parser: {
				'javascript/auto': {
					overrideStrict: "strict"
				}
			}
		}
	},
	{
		mode: "production",
		entry: ["./strict"],
		module: {
			parser: {
				'javascript/auto': {
					overrideStrict: "non-strict"
				}
			}
		}
	}
];
