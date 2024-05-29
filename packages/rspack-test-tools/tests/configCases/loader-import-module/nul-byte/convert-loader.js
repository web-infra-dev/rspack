const path = require('path');

module.exports = async function () {
  const importLoader = path.resolve(__dirname, 'import-loader.js');
  const sourceLoader = path.resolve(__dirname, 'source-loader.js');
  const empty = path.resolve(__dirname, 'empty.js');
  return `
    import value from '${importLoader}!${sourceLoader}?{"content": "#b"}!${empty}';
    export default value;`
    ;
}