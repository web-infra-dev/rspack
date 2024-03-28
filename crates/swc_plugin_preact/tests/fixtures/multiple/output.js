import { createContext as cc } from 'preact';
import * as ns from 'preact';
import df from 'preact';
export function aaa(a, b) {
  Object.assign(cc[`__file_hash__1_${a}${b}`] || (cc[`__file_hash__1_${a}${b}`] = cc({})), {
    __: {}
  });
  ns.createContext[`__file_hash__2_${a}${b}`] || (ns.createContext[`__file_hash__2_${a}${b}`] = ns.createContext());
  Object.assign(df.createContext[`__file_hash__3_${a}${b}`] || (df.createContext[`__file_hash__3_${a}${b}`] = df.createContext(b)), {
    __: b
  });
  return function bbb(a, b, c) {
    Object.assign(cc[`__file_hash__4_${a}${b}${c}`] || (cc[`__file_hash__4_${a}${b}${c}`] = cc({})), {
      __: {}
    });
    ns.createContext[`__file_hash__5_${a}${b}${c}`] || (ns.createContext[`__file_hash__5_${a}${b}${c}`] = ns.createContext());
    Object.assign(df.createContext[`__file_hash__6_${a}${b}${c}`] || (df.createContext[`__file_hash__6_${a}${b}${c}`] = df.createContext(b)), {
      __: b
    });
  };
}
