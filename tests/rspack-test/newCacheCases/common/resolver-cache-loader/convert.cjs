<delete>
---
<delete>
---
module.exports = function (source) {
  return `export default ${JSON.stringify(`cjs:${source.trim()}`)}`;
};
---
<delete>
