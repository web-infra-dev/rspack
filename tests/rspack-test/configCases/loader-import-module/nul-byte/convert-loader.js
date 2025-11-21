const path = require('path');

module.exports = async function () {
  const importLoader = path.resolve(__dirname, 'import-loader.js');
  const sourceLoader = path.resolve(__dirname, 'source-loader.js');
  const empty = path.resolve(__dirname, 'empty.js');
  const request = `${importLoader}!${sourceLoader}?{"content": "#b"}!${empty}`;
  return `
    import value from ${JSON.stringify(request)};
    export default value;`
    ;
}