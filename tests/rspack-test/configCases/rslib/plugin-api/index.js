console.log(require.cache)
console.log(require.extensions)
console.log(require.config)
console.log(require.version)
console.log(require.include)
console.log(require.onError)
export function g() {
  console.log(module);
  if (module.children) {
    module.children = module.children.filter((item) => item.filename !== path);
  }
}


