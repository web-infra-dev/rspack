module.exports = function (content) {
  var idx = this.getOptions().updateIndex;
  var items = content.split(/---+\r?\n/g);
  this.callback(null, items[idx] || items[items.length - 1])
}