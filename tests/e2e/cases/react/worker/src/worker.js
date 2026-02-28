import Button from './Button';

self.onmessage = () => {
  Button.add();
  postMessage(Button.get());
};
