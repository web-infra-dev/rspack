// should receive warning from css-loader,
// however we don't support loaderContext._module,
// css-loader uses loaderContext._module.type to
// decide bailout on builtin css module

module.exports = `WARNING in ./style.css
  ⚠ You can't use \`experiments.css\` and \`css-extract-rspack-plugin\` together, please set \`experiments.css\` to \`false\``
