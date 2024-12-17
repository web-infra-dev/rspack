"use strict";

// refactor: replace Regex in consumesCommon.js
// Original source code str.trim().split(/(?<=[-0-9A-Za-z])\s+/g);

const orgFunc = (str) => str.trim().split(/(?<=[-0-9A-Za-z])\s+/g);

const newFunc = (str) => str.trim().split('').reduce((pre, cur) => {
  const lastIdx = pre.length-1;
  const t = pre[lastIdx] || '';
  if(/\s/.test(cur) && /[-0-9A-Za-z]/.test(t[t.length-1])){
    pre.push('')
  }else{
    pre[lastIdx] = t+cur
  }
  return pre
}, ['']).filter(t=>t)

describe("ConsumesCommonRegex", () => {
  const cases = [
    "5 || 6 || 7.x.x",
    "1 - 2",
    "=3",
    "=3.0",
    "^3.4",
    "3.4 - 6.5",
    "<=3.4",
    ">3.4",
    "1.2.3-alpha.x.x",
    "1.2.3-NaN"
  ];  
  for(let i=0; i<cases.length; i++){
    const str = cases[i];
    it(`replace Regex in consumesCommon.js: ${str}`, () => {
      expect(newFunc(str)).toStrictEqual(orgFunc(str));
    });
  }
});
