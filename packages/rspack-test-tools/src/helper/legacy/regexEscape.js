// @ts-nocheck
'use strict';

export function regexEscape(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); // $& means the whole matched string
}
