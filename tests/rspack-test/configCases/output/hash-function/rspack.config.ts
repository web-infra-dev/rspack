import { defineConfig } from '@rspack/cli';
import { createHash } from 'node:crypto';

class SHA256Hash {
  private _hash = createHash('sha256');

  update(
    data: string | Buffer,
    inputEncoding?: import('node:crypto').Encoding,
  ) {
    if (typeof data === 'string') {
      if (inputEncoding) this._hash.update(data, inputEncoding);
      else this._hash.update(data);
    } else {
      this._hash.update(data);
    }
    return this;
  }

  digest(encoding?: import('node:crypto').BinaryToTextEncoding) {
    return encoding ? this._hash.digest(encoding) : this._hash.digest();
  }
}

export default defineConfig({
  output: {
    hashDigest: 'base64',
    hashDigestLength: 25,
    // @ts-expect-error Rspack does not support custom hash constructors yet.
    hashFunction: SHA256Hash,
    hashSalt: 'salt',
  },
});
