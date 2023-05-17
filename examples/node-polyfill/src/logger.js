module.exports = function logger(method, ...args) {
	console[method](...args);
};
