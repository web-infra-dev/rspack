/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		output: {
			hashFunction: require("xxhashjs").h32
		}
	}
];
