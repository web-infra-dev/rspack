const scriptsWithIntegrity = [];

const observer = new MutationObserver((mutationsList) => {
  Array.from(mutationsList).forEach((mutation) => {
    Array.from(mutation.addedNodes || []).forEach((node) => {
      if (node.nodeName === "SCRIPT") {
        if (
          node.getAttribute("crossOrigin") === "anonymous" &&
          node
            .getAttribute("integrity")
            .match(/^sha256-[-A-Za-z0-9+/=]{44} sha384-[-A-Za-z0-9+/=]{64}$/)
        ) {
          scriptsWithIntegrity.push(node);
        }
      }
    });
  });
});

observer.observe(document.querySelector("head"), { childList: true });

import("./chunk")
  .then(() => {
    if (
      scriptsWithIntegrity.some(
        (script) =>
          new URL(script.getAttribute("src")).pathname === "/chunk_js.js"
      )
    ) {
      console.log("ok");
    } else {
      console.log("error");
    }
  })
  .catch((e) => {
    console.error(e);
    console.log("error");
  });
