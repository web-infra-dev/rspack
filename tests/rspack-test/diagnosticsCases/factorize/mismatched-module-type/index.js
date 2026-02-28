it("should throw if `jsx` is used in `js`", () => {
  let errored = false
  try {
	  require("./app.js");
  } catch(err) {
    errored = true
  }
  expect(errored).toBeTruthy()
});

it("should throw if `tsx` is used in `ts`", () => {
  let errored = false
  try {
	  require("./app.ts");
  } catch(err) {
    errored = true
  }
  expect(errored).toBeTruthy()
});

it("should throw if `jsx` is sat to type `js`", () => {
  let errored = false
  try {
	  require("./app.jsx");
  } catch(err) {
    errored = true
  }
  expect(errored).toBeTruthy()
})

it("should throw if `tsx` is sat to type `ts`", () => {
  let errored = false
  try {
	  require("./app.tsx");
  } catch(err) {
    errored = true
  }
  expect(errored).toBeTruthy()
})