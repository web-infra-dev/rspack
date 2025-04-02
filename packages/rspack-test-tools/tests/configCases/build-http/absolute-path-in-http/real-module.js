// This is a real module that should be imported from disk
const message = "Hello from real module!";

function getMessage() {
  return message;
}

// Support dual export for both CommonJS and ESM
// ESM exports
export { message, getMessage };

// CommonJS exports
module.exports = {
  message,
  getMessage
};
