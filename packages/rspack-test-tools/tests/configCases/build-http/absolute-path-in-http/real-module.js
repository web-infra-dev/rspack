// This is a real module that should be imported from disk
const message = "Hello from real module!";

function getMessage() {
  return message;
}

module.exports = {
  message,
  getMessage
};
