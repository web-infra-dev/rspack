
/*
module.exports = function (config) {
	// In node 10 v8 has a bug which inserts an additional micro-tick into async functions
	return !process.version.startsWith("v10.");
};

*/
module.exports = () => {return "https://github.com/web-infra-dev/rspack/issues/3889"}

							