// @ts-nocheck
const contentMap = {};
module.exports = function (content) {
	// CHANGE:
	var idx = this.getOptions().updateIndex;
	var items = content.split(/---+\r?\n/g);
	var curIdx = items[idx] ? idx : items.length - 1;
	var oldIdx = contentMap[this.resourcePath];
	if (curIdx !== oldIdx && global.__CHANGED_FILES__) {
		global.__CHANGED_FILES__.set(this.resourcePath, items.length);
	}
	contentMap[this.resourcePath] = curIdx;
	if (items.length > 1) {
		this.cacheable(false);
	}
	this.callback(null, items[curIdx]);
};
