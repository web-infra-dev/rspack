// @ts-nocheck
import { URL } from 'node:url';

function urlToRelativePath(url) {
  if (url.startsWith('https://') || url.startsWith('file://')) {
    const urlObj = new URL(url);
    return `./${urlObj.pathname.split('/').pop()}`;
  }
  return `./${url}`;
}

export { urlToRelativePath };
