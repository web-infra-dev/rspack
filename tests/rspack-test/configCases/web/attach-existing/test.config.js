module.exports = {
  resourceLoader: function (url) {
    // ignore /somewhere/else.js
    if (/\/somewhere\/else\.js$/.test(url)) {
      return null;
    }
  }
};