'use client';

import { type ReactNode, useRef } from 'react';

export function Dialog({
  trigger,
  children,
}: {
  trigger: ReactNode;
  children: ReactNode;
}) {
  const ref = useRef<HTMLDialogElement | null>(null);
  return (
    <>
      <button type="button" onClick={() => ref.current?.showModal()}>
        {trigger}
      </button>
      <dialog ref={ref} onSubmit={() => ref.current?.close()}>
        {children}
      </dialog>
    </>
  );
}
