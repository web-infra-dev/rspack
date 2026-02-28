/**
 * The following code is modified based on
 * https://github.com/suguru03/neo-async/blob/master/lib/async.js
 *
 * MIT Licensed
 * Author Suguru Motegi
 * Copyright (c) 2014-2018 Suguru Motegi
 * https://github.com/suguru03/neo-async/blob/master/LICENSE
 */
export interface Dictionary<T> {
  [key: string]: T;
}
export type IterableCollection<T> = T[] | IterableIterator<T> | Dictionary<T>;
export type ErrorCallback<E = Error> = (err?: E | null) => void;
export type AsyncIterator<T, E = Error> = (
  item: T,
  callback: ErrorCallback<E>,
) => void;

function throwError() {
  throw new Error('Callback was already called.');
}

function noop() {}

function onlyOnce<E = Error>(func: ErrorCallback<E>): ErrorCallback<E> {
  return (err?: E | null) => {
    const fn = func;
    func = throwError;
    fn(err);
  };
}

function once<E = Error>(func: ErrorCallback<E>): ErrorCallback<E> {
  return (err?: E | null) => {
    const fn = func;
    func = noop;
    fn(err);
  };
}

function arrayEach<T, E = Error>(
  array: T[],
  iterator: (item: T, callback: ErrorCallback<E>) => void,
  callback: ErrorCallback<E>,
): void {
  let index = -1;
  while (++index < array.length) {
    iterator(array[index], onlyOnce(callback));
  }
}

/**
 * @example
 *
 * // array
 * var order = [];
 * var array = [1, 3, 2];
 * var iterator = function(num, done) {
 *   setTimeout(function() {
 *     order.push(num);
 *     done();
 *   }, num * 10);
 * };
 * asyncLib.each(array, iterator, function(err, res) {
 *   console.log(res); // undefined
 *   console.log(order); // [1, 2, 3]
 * });
 *
 * @example
 *
 * // break
 * var order = [];
 * var array = [1, 3, 2];
 * var iterator = function(num, done) {
 *   setTimeout(function() {
 *     order.push(num);
 *     done(null, num !== 2);
 *   }, num * 10);
 * };
 * asyncLib.each(array, iterator, function(err, res) {
 *   console.log(res); // undefined
 *   console.log(order); // [1, 2]
 * });
 *
 */
function each<T, E = Error>(
  collection: IterableCollection<T>,
  iterator: AsyncIterator<T, E>,
  originalCallback: ErrorCallback<E>,
) {
  let callback = once(originalCallback);
  let size = 0;
  let completed = 0;

  const done: ErrorCallback<E> = (err) => {
    if (err) {
      callback = once(callback);
      callback(err);
    } else if (++completed === size) {
      callback(null);
    }
  };

  if (Array.isArray(collection)) {
    size = collection.length;
    arrayEach(collection, iterator, done);
  }
  if (!size) {
    callback(null);
  }
}

export default { each };
