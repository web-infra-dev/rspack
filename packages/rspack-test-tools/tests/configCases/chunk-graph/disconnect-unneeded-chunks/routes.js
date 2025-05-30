export const value = import(/*webpackChunkName: "foo"*/ './foo.js').then(({ value }) => value)
