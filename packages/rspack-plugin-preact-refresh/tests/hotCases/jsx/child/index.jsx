import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './app';
import update from '../../update';

const container = document.createElement('div');
container.id = "root";
document.body.appendChild(container);
const root = ReactDOM.createRoot(container);
root.render(
  <div>
    <App />
  </div>,
);

it("should rerender when children change", (done) => {
  expect(container.querySelector('span').textContent).toBe("no child");
  NEXT(
    update(done, true, () => {
      expect(container.querySelector('span').textContent).toBe("has child");
      NEXT(
        update(done, true, () => {
          expect(container.querySelector('span').textContent).toBe("child change");
          done()
        }),
      )
    }),
  )
});


if (module.hot) {
  module.hot.accept('./app');
}
