let value = 1;
const self = require("./module.js");

export { value };
export const increment = () => value++;
export const requireSeesEsModule = () => self.__esModule;
export default 42;
