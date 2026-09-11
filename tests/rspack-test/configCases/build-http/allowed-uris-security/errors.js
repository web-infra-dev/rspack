module.exports = [
  /http:\/\/allowed\.example@blocked\.example\/userinfo\.js doesn't match the allowedUris policy/,
  /http:\/\/allowed\.example\.blocked\.example\/host-prefix\.js doesn't match the allowedUris policy/,
  /http:\/\/path\.example\/outside\.js doesn't match the allowedUris policy/,
  /https:\/\/blocked\.example\/invalid-rule\.js doesn't match the allowedUris policy/,
  /http:\/\/allowed\.example@blocked\.example\/redirected\.js doesn't match the allowedUris policy after redirect/,
];
