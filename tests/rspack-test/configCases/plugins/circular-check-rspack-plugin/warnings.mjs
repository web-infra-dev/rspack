export default [
  {
    compilerPath: /^detects-basic-cycle$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]b\.js -> \.[\\/]deps[\\/]c\.js -> \.[\\/]deps[\\/]b\.js/,
  },
  {
    compilerPath: /^detects-deep-cycle$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]e\.js -> \.[\\/]deps[\\/]f\.js -> \.[\\/]deps[\\/]g\.js -> \.[\\/]deps[\\/]e\.js/,
  },
  {
    compilerPath: /^include$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]e\.js -> \.[\\/]deps[\\/]f\.js -> \.[\\/]deps[\\/]g\.js -> \.[\\/]deps[\\/]e\.js/,
  },
  {
    compilerPath: /^on-detected-can-report$/,
    message: /\.[\\/]deps[\\/]e\.js -> \.[\\/]deps[\\/]f\.js -> \.[\\/]deps[\\/]g\.js -> \.[\\/]deps[\\/]e\.js/,
  },
  {
    compilerPath: /^module-concat-plugin-compat$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]module-concat-plugin-compat[\\/]a\.js -> \.[\\/]deps[\\/]module-concat-plugin-compat[\\/]b\.js -> \.[\\/]deps[\\/]module-concat-plugin-compat[\\/]a\.js/,
  },
  {
    compilerPath: /^ignores-self-import$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]self-referencing[\\/]imports-self\.js -> \.[\\/]deps[\\/]self-referencing[\\/]imports-self\.js/,
  },
  {
    compilerPath: /^detects-esm-self-import$/,
    message: /Circular dependency detected:[\s\S]*\.[\\/]deps[\\/]self-referencing[\\/]esm-imports-self\.js -> \.[\\/]deps[\\/]self-referencing[\\/]esm-imports-self\.js/,
  },
];
