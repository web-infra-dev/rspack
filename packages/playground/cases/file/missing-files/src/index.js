// @ts-nocheck
import { a } from "./missing-file-1";
const { b } = require("./missing-file-2");

document.getElementById("root").innerHTML = `
<span id="missing-file-1">${a}</span>
<span id="missing-file-2">${b}</span>
`;
