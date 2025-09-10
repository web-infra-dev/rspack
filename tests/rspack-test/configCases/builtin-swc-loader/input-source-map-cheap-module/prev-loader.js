/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function() {
	// Return the pre-transformed code and sourcemap
	this.callback(null, `export function a0() {
  a2('*a0*');
  return /*#__PURE__*/React.createElement("view", null);
}
function A1() {
  return /*#__PURE__*/React.createElement("view", null, '*a1*');
}
export function a2() {
  return /*#__PURE__*/React.createElement("view", null, /*#__PURE__*/React.createElement(A1, {
    bar: '*a2*'
  }));
}`, {
  version: 3,
  file: undefined,
  names: [ 'a0', 'a2', 'React', 'createElement', 'A1', 'bar' ],
  sourceRoot: undefined,
  sources: [
    this.resourcePath
  ],
  sourcesContent: [
    'export function a0() {\n' +
      "  a2('*a0*')\n" +
      '  return <view></view>\n' +
      '}\n' +
      '\n' +
      'function A1() {\n' +
      "  return <view>{'*a1*'}</view>\n" +
      '}\n' +
      '\n' +
      'export function a2() {\n' +
      '  return (\n' +
      '    <view>\n' +
      "      <A1 bar={'*a2*'} />\n" +
      '    </view>\n' +
      '  )\n' +
      '}\n' +
      '\n'
  ],
  mappings: 'AAAA,OAAO,SAASA,EAAEA,CAAA,EAAG;EACnBC,EAAE,CAAC,MAAM,CAAC;EACV,oBAAOC,KAAA,CAAAC,aAAA,aAAY,CAAC;AACtB;AAEA,SAASC,EAAEA,CAAA,EAAG;EACZ,oBAAOF,KAAA,CAAAC,aAAA,eAAO,MAAa,CAAC;AAC9B;AAEA,OAAO,SAASF,EAAEA,CAAA,EAAG;EACnB,oBACEC,KAAA,CAAAC,aAAA,4BACED,KAAA,CAAAC,aAAA,CAACC,EAAE;IAACC,GAAG,EAAE;EAAO,CAAE,CACd,CAAC;AAEX',
  ignoreList: []
})
}