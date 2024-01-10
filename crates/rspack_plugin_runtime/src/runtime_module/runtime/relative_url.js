__webpack_require__.U = function RelativeURL(url) {
  var realUrl = new URL(url, "x:/");
  var values = {};
  for (var key in realUrl) values[key] = realUrl[key];
  values.href = url;
  values.pathname = url.replace(/[?#].*/, "");
  values.origin = values.protocol = "";
  values.toString = values.toJSON = function () {
    return url;
  };
  for (var key in values) Object.defineProperty(this, key, { enumerable: true, configurable: true, value: values[key] });
};
__webpack_require__.U.prototype = URL.prototype;