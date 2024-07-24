// should receive warning from css-loader,
// however we don't support loaderContext._module,
// css-loader uses loaderContext._module.type to
// decide bailout on builtin css module

module.exports = `WARNING in ./style.css
  ⚠ use type 'css' and \`CssExtractRspackPlugin\` together, please set \`experiments.css\` to \`false\` or set \`{ type: "javascript/auto" }\` for rules with \`CssExtractRspackPlugin\` in your rspack config (now \`CssExtractRspackPlugin\` does nothing). 
 @ ./style.css

WARNING in ./style.css
  ⚠ ModuleWarning: You can't use \`experiments.css\` (\`experiments.futureDefaults\` enable built-in CSS support by default) and \`css-loader\` together, please set \`experiments.css\` to \`false\` or set \`{ type: "javascript/auto" }\` for rules with \`css-loader\` in your webpack config (now css-loader does nothing). 
                 @ ./style.css`;