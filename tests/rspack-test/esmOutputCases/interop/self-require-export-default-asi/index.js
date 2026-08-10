let works
const self = require("./index.js")
works = self.__esModule === true
export default 123

it("keeps an ASI boundary before a rewritten default export", () => {
	expect(works).toBe(true)
	expect(self.default).toBe(123)
})
