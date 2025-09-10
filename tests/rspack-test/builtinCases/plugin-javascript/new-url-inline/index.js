const imgSrc = new URL("./react.svg", import.meta.url);
const imgSrc2 = require("./vue.svg");
const img = new Image();
img.src = imgSrc.href;
img.src = imgSrc2;
