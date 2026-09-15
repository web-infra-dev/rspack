// @ts-nocheck
const URL = require('url').URL;

function urlToRelativePath(url) {
  if (url.startsWith('https://') || url.startsWith('file://')) {
    const urlObj = new URL(url);
    return `./${urlObj.pathname.split('/').pop()}`;
  }
  return `./${url}`;
}

export { urlToRelativePath };
