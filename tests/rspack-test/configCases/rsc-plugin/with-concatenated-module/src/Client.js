'use client';

import { add, del, get, update } from './actions';

export const Client = () => {
  async function onClick() {
    await add();
    await del();
    await get();
    await update();
  }

  return (
    <button type="button" onClick={onClick}>
      Run actions
    </button>
  );
};
