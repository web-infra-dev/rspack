import { useState } from 'react';

interface MenuGroupProps {
  children: React.ReactNode;
  defaultLabel: string;
}

function Down() {
  return (
    <svg width="1em" height="1em" viewBox="0 0 32 32">
      <path
        fill="currentColor"
        d="M16 22L6 12l1.4-1.4l8.6 8.6l8.6-8.6L26 12z"
      ></path>
    </svg>
  );
}

export function MenuGroup({ children, defaultLabel }: MenuGroupProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div
      className="relative h-14 flex flex-center"
      onMouseLeave={() => setIsOpen(false)}
    >
      <button
        onMouseEnter={() => setIsOpen(true)}
        className="flex-center text-sm text-1 hover:text-text-2 font-medium transition-colors duration-200"
      >
        <span className="font-medium text-base">{defaultLabel}</span>
        <Down />
      </button>
      <div
        className="absolute top-10 mx-0.8 transition-opacity duration-300"
        style={{
          opacity: isOpen ? 1 : 0,
          visibility: isOpen ? 'visible' : 'hidden',
        }}
      >
        <div
          className="p-3 w-full h-full w-auto max-h-100vh rounded-xl whitespace-nowrap bg-white"
          style={{
            boxShadow: 'var(--rp-shadow-3)',
            marginRight: '-1.5rem',
            zIndex: 100,
            border: '1px solid var(--rp-c-divider-light)',
          }}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
