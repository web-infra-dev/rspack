let count = 0;
module.exports = {
  resourceLoader: function (url) {
    // should failed 2 times
    if (/the-chunk\.js$/.test(url)) {
      if (count < 2) {
        count++;
        return null;
      }
    }
  }
};