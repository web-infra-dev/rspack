// Keep b reachable without making it available in the outer chunk.
module.exports = () => new Promise((resolve, reject) => {
	require.ensure(["./b", "./c"], () => {
		resolve([require("./b"), require("./c")]);
	}, error => reject(error), "other");
});
