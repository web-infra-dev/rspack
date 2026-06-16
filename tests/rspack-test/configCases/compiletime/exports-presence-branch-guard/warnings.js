module.exports = [
  {
    moduleName: /index/,
    message: /export 'e'.*was not found in '\.\/foo'/,
    loc: /7:32-37/,
  },
  {
    moduleName: /index/,
    message: /export 'obj'\.'e'.*was not found in '\.\/foo'/,
    loc: /11:37-42/,
  },
  {
    moduleName: /index/,
    message: /export 'obj'\.'e'.*was not found in '\.\/foo'/,
    loc: /14:42-51/,
  },
  {
    moduleName: /webpack-20561/,
    message: /export 'a'.*was not found in '\.\/stub'/,
    loc: /31:6-10/,
  },
  {
    moduleName: /webpack-20561/,
    message: /export 'b'.*was not found in '\.\/stub'/,
    loc: /32:6-10/,
  },
  {
    moduleName: /webpack-20561/,
    message: /export 'a'.*was not found in '\.\/stub'/,
    loc: /54:5-9/,
  },
  {
    moduleName: /webpack-20561/,
    message: /export 'a'.*was not found in '\.\/stub'/,
    loc: /55:15-19/,
  },
  {
    moduleName: /webpack-20561/,
    message: /export 'a'.*was not found in '\.\/stub'/,
    loc: /57:5-9/,
  },
  {
    moduleName: /webpack-20561/,
    message: /export 'a'.*was not found in '\.\/stub'/,
    loc: /58:15-19/,
  },
];
