import { h, render } from "preact";
import { App } from "./app";

const container = document.createElement("div");
document.body.appendChild(container);
render(h(App), container);

it("refreshes Preact components and retains their state", async () => {
  const button = () => container.querySelector("button");
  expect(button().textContent).toBe("before: 1");
  button().click();
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(button().textContent).toBe("before: 2");
  await NEXT_HMR();
  await new Promise(resolve => setTimeout(resolve, 0));
  expect(button().textContent).toBe("after: 2");
  render(null, container);
  container.remove();
});
