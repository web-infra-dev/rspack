['en.js', 'zh.js'].forEach(local => {
  require("./locals/" + local);
})

require("./globalIndex.js");
