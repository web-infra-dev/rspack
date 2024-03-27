import pp from 'preact';
export function aaa(a, b) {
  const context = pp.createContext[`__file_hash__$context1_${a}${b}`] || (pp.createContext[`__file_hash__$context1_${a}${b}`] = pp.createContext());
}