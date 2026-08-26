let cacheable = true;

module.exports = function loader(source) {
	this.cacheable(cacheable);
	return source;
};

module.exports.setCacheable = value => {
	cacheable = value;
};
