// Current modern-module export deconfliction uses the quoted export name as the
// temporary local identifier base, which emits invalid JS for names like `'x`.
module.exports = () =>
  "TODO: quoted string export names that start with a quote generate invalid identifiers";
