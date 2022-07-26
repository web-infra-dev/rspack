function load_style(chunkId, href, fullhref, resolve, reject) {
  var existingLinkTags = document.getElementsByTagName('link');
  for (var i = 0; i < existingLinkTags.length; i++) {
    var tag = existingLinkTags[i];
    var dataHref = tag.getAttribute('data-href') || tag.getAttribute('href');
    if (tag.rel === 'stylesheet' && (dataHref === href || dataHref === fullhref)) return resolve();
  }
  var existingStyleTags = document.getElementsByTagName('style');
  for (var i = 0; i < existingStyleTags.length; i++) {
    var tag = existingStyleTags[i];
    var dataHref = tag.getAttribute('data-href');
    if (dataHref === href || dataHref === fullhref) return resolve();
  }
  var linkTag = document.createElement('link');
  linkTag.rel = 'stylesheet';
  linkTag.type = 'text/css';
  var onLinkComplete = function (event) {
    linkTag.onerror = linkTag.onload = null;
    if (event.type === 'load') {
      resolve();
    } else {
      var errorType = event && (event.type === 'load' ? 'missing' : event.type);
      var realHref = (event && event.target && event.target.href) || fullhref;
      var err = new Error('Loading CSS chunk ' + chunkId + ' failed.\n(' + realHref + ')');
      err.code = 'CSS_CHUNK_LOAD_FAILED';
      err.type = errorType;
      err.request = realHref;
      linkTag.parentNode.removeChild(linkTag);
      reject(err);
    }
  };
  linkTag.onerror = linkTag.onload = onLinkComplete;
  linkTag.href = fullhref;
  document.head.appendChild(linkTag);
  return linkTag;
}

function __rspack_load_dynamic_css__(chunkId, promises) {
  var installedChunkData = this.installedCssChunks[chunkId];
  if (installedChunkData) {
    promises.push(installedChunkData);
  } else if (installedChunkData !== 0 && this.__rspack_has_dynamic_chunk__(chunkId, 'css')) {
    var href = this.__rspack_get_dynamic_chunk_url__(chunkId, 'css');
    var fullhref = this.publicPath + href;
    promises.push(
      (installedChunkData = new Promise(function (resolve, reject) {
        load_style(chunkId, href, fullhref, resolve, reject);
      }).then(
        function () {
          installedChunkData = 0;
        },
        function (e) {
          delete installedChunkData;
          throw e;
        }
      ))
    );
  }
}

// mount load dynamic css
(function () {
  runtime.__rspack_load_dynamic_css__ = __rspack_load_dynamic_css__;
})();