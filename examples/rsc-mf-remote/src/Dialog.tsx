'use client';

import { type ReactNode, useRef } from 'react';

export function Dialog({
  trigger,
  children,
  buttonTestId,
}: {
  trigger: ReactNode;
  children: ReactNode;
  buttonTestId?: string;
}) {
  const ref = useRef<HTMLDialogElement | null>(null);
  return (
    <>
      <button
        type="button"
        data-testid={buttonTestId}
        onClick={() => ref.current?.showModal()}
      >
        {trigger}
      </button>
      <dialog ref={ref} onSubmit={() => ref.current?.close()}>
        {children}
      </dialog>
    </>
  );
}
