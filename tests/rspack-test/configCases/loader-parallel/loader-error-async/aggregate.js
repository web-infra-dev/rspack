module.exports = function () {
	throw new AggregateError([42, "rejected"], "Failed to load (aggregate)");
};
