module.exports = {
	entry: {
		a1: "./a",
		b1: {
			runtime: "a1",
			import: "./b"
		}
	}
};
