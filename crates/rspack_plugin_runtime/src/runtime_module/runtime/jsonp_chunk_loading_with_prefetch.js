__webpack_require__.F.j = function (chunkId) {
  if ((!__webpack_require__.o(installedChunks, chunkId) || installedChunks[chunkId] === undefined) && $JS_MATCHER$) {
    installedChunks[chunkId] = null;
    var link = document.createElement('link');
    $CROSS_ORIGIN$
    if (__webpack_require__.nc) {
      link.setAttribute("nonce", __webpack_require__.nc);
    }
    link.rel = "prefetch";
    link.as = "script";
    link.href = __webpack_require__.p + __webpack_require__.u(chunkId);
    document.head.appendChild(link);
  }
};
