module.exports = function (content) {
	// CHANGE:
	var idx = this.getOptions().updateIndex;
	var items = content.split(/---+\r?\n/g);
	if (items.length > 1) {
		this.cacheable(false);
	}
	this.callback(null, items[idx] || items[items.length - 1]);
};
