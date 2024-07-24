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

it("should keep state", (done) => {
  expect(container.querySelector('span').textContent).toBe("before: dark");
  NEXT(
    update(done, true, () => {
      expect(container.querySelector('span').textContent).toBe("after: dark");
      done();
    }),
  )
});


if (module.hot) {
  module.hot.accept('./app');
}
