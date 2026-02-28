import('./main');

import('./a').then(({ a }) => {
    a();
});

import('./b').then(({ b }) => {
    b();
});
