function component () {
	return <div></div>
}

it('should work with react refresh', () => {
  const fs = require("fs")
  const map = fs.readFileSync(__filename + ".map", "utf-8")
  expect(map).toContain(STUB)
})
