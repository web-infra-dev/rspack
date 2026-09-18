import 'http://allowed.example@blocked.example/userinfo.js';
import 'http://allowed.example.blocked.example/host-prefix.js';
import 'http://path.example/modules/../outside.js';
import 'https://blocked.example/invalid-rule.js';
import 'http://allowed.example/redirect';

throw new Error('Compilation should reject disallowed URLs');
