import { createContext } from 'preact';
export function aaa(a, b) {
  const context = createContext[`__file_hash__$context1_${a}${b}`] || (createContext[`__file_hash__$context1_${a}${b}`] = createContext());
}