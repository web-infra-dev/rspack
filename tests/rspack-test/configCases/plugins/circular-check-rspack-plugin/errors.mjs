export default [
  {
    compilerPath: /^fail-on-error$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]e\.js -> \.[\\/]deps[\\/]f\.js -> \.[\\/]deps[\\/]g\.js -> \.[\\/]deps[\\/]e\.js/,
  },
];
