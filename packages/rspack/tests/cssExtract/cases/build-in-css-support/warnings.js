// should receive warning from css-loader,
// however we don't suppot loaderContext._module,
// css-loader uses loaderContext._module.type to
// decide bailout on builtin css module

module.exports = `WARNING in ⚠ You can't use \`experiments.css\` and \`mini-css-extract-plugin\` together, please set \`experiments.css\` to \`false\``;
