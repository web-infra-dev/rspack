'use strict';

export function supportsTextDecoder() {
  try {
    return typeof TextDecoder !== 'undefined';
  } catch (_err) {
    return false;
  }
}
