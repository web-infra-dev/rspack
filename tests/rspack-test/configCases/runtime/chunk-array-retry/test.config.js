const failed = new Set();
module.exports = {
  findBundle(index) { return 'main-' + index + '.js'; },
  resourceLoader(url) {
    if (/(js-group-0\.js|css-group-1\.css)$/.test(url) && !failed.has(url)) {
      failed.add(url);
      return null;
    }
  },
};
