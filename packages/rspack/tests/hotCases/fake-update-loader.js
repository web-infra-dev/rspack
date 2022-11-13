module.exports = function (context) {
  var idx = this.getOptions().updateIndex;
  var items = context.source.getCode().split(/---+\r?\n/g);
  return {
		content: items[idx] || items[items.length - 1],
		meta: "",
		sourceMap: "{}"
	};
}