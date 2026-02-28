async function pMap(iterable, mapper, {
  concurrency = Number.POSITIVE_INFINITY,
  stopOnError = true,
  signal
} = {}) {
  return new Promise((resolve_, reject_) => {
    if (iterable[Symbol.iterator] === void 0 && iterable[Symbol.asyncIterator] === void 0) {
      throw new TypeError(`Expected \`input\` to be either an \`Iterable\` or \`AsyncIterable\`, got (${typeof iterable})`);
    }
    if (typeof mapper !== "function") {
      throw new TypeError("Mapper function is required");
    }
    if (!(Number.isSafeInteger(concurrency) && concurrency >= 1 || concurrency === Number.POSITIVE_INFINITY)) {
      throw new TypeError(`Expected \`concurrency\` to be an integer from 1 and up or \`Infinity\`, got \`${concurrency}\` (${typeof concurrency})`);
    }
    const result = [];
    const errors = [];
    const skippedIndexesMap = new Map();
    let isRejected = false;
    let isResolved = false;
    let isIterableDone = false;
    let resolvingCount = 0;
    let currentIndex = 0;
    const iterator = iterable[Symbol.iterator] === void 0 ? iterable[Symbol.asyncIterator]() : iterable[Symbol.iterator]();
    const signalListener = () => {
      reject(signal.reason);
    };
    const cleanup = () => {
      signal == null ? void 0 : signal.removeEventListener("abort", signalListener);
    };
    const resolve = (value) => {
      resolve_(value);
      cleanup();
    };
    const reject = (reason) => {
      isRejected = true;
      isResolved = true;
      reject_(reason);
      cleanup();
    };
    if (signal) {
      if (signal.aborted) {
        reject(signal.reason);
      }
      signal.addEventListener("abort", signalListener, {once: true});
    }
    const next = async () => {
      if (isResolved) {
        return;
      }
      const nextItem = await iterator.next();
      const index = currentIndex;
      currentIndex++;
      if (nextItem.done) {
        isIterableDone = true;
        if (resolvingCount === 0 && !isResolved) {
          if (!stopOnError && errors.length > 0) {
            reject(new AggregateError(errors));
            return;
          }
          isResolved = true;
          if (skippedIndexesMap.size === 0) {
            resolve(result);
            return;
          }
          const pureResult = [];
          for (const [index2, value] of result.entries()) {
            if (skippedIndexesMap.get(index2) === pMapSkip) {
              continue;
            }
            pureResult.push(value);
          }
          resolve(pureResult);
        }
        return;
      }
      resolvingCount++;
      (async () => {
        try {
          const element = await nextItem.value;
          if (isResolved) {
            return;
          }
          const value = await mapper(element, index);
          if (value === pMapSkip) {
            skippedIndexesMap.set(index, value);
          }
          result[index] = value;
          resolvingCount--;
          await next();
        } catch (error) {
          if (stopOnError) {
            reject(error);
          } else {
            errors.push(error);
            resolvingCount--;
            try {
              await next();
            } catch (error2) {
              reject(error2);
            }
          }
        }
      })();
    };
    (async () => {
      for (let index = 0; index < concurrency; index++) {
        try {
          await next();
        } catch (error) {
          reject(error);
          break;
        }
        if (isIterableDone || isRejected) {
          break;
        }
      }
    })();
  });
}
function pMapIterable(iterable, mapper, {
  concurrency = Number.POSITIVE_INFINITY,
  backpressure = concurrency
} = {}) {
  if (iterable[Symbol.iterator] === void 0 && iterable[Symbol.asyncIterator] === void 0) {
    throw new TypeError(`Expected \`input\` to be either an \`Iterable\` or \`AsyncIterable\`, got (${typeof iterable})`);
  }
  if (typeof mapper !== "function") {
    throw new TypeError("Mapper function is required");
  }
  if (!(Number.isSafeInteger(concurrency) && concurrency >= 1 || concurrency === Number.POSITIVE_INFINITY)) {
    throw new TypeError(`Expected \`concurrency\` to be an integer from 1 and up or \`Infinity\`, got \`${concurrency}\` (${typeof concurrency})`);
  }
  if (!(Number.isSafeInteger(backpressure) && backpressure >= concurrency || backpressure === Number.POSITIVE_INFINITY)) {
    throw new TypeError(`Expected \`backpressure\` to be an integer from \`concurrency\` (${concurrency}) and up or \`Infinity\`, got \`${backpressure}\` (${typeof backpressure})`);
  }
  return {
    async *[Symbol.asyncIterator]() {
      const iterator = iterable[Symbol.asyncIterator] === void 0 ? iterable[Symbol.iterator]() : iterable[Symbol.asyncIterator]();
      const promises = [];
      let runningMappersCount = 0;
      let isDone = false;
      let index = 0;
      function trySpawn() {
        if (isDone || !(runningMappersCount < concurrency && promises.length < backpressure)) {
          return;
        }
        const promise = (async () => {
          const {done, value} = await iterator.next();
          if (done) {
            return {done: true};
          }
          runningMappersCount++;
          trySpawn();
          try {
            const returnValue = await mapper(await value, index++);
            runningMappersCount--;
            if (returnValue === pMapSkip) {
              const index2 = promises.indexOf(promise);
              if (index2 > 0) {
                promises.splice(index2, 1);
              }
            }
            trySpawn();
            return {done: false, value: returnValue};
          } catch (error) {
            isDone = true;
            return {error};
          }
        })();
        promises.push(promise);
      }
      trySpawn();
      while (promises.length > 0) {
        const {error, done, value} = await promises[0];
        promises.shift();
        if (error) {
          throw error;
        }
        if (done) {
          return;
        }
        trySpawn();
        if (value === pMapSkip) {
          continue;
        }
        yield value;
      }
    }
  };
}
const pMapSkip = Symbol("skip");
export default pMap;
export {pMapIterable, pMapSkip};
