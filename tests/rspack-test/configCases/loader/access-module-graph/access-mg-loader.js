module.exports = function loader(content) {
  
  const moduleGraph = this._compilation.moduleGraph;
  let issuer = moduleGraph.getIssuer(this._module);
  console.log(issuer);
  return content;
}