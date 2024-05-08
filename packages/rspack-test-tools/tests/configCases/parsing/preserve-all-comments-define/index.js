const fs = require("fs");

async function renderInBrowser() {
	const jsx = () => {};
	const RootApp = {};
  if (process.env.__IS_REACT_18__) {
    /* @__PURE__ */ jsx(RootApp, {});
  } else {
    /* @__PURE__ */ jsx(RootApp, {});
  }
}
renderInBrowser();

it("should only contain a single pure annotation after being minimized", () => {})
