'use client';

export default function SharedComponentClientComponent({
  testId = 'remote-shared-rsc-probe',
  label = 'shared-rsc-probe',
}) {
  return <p data-testid={testId}>{label}</p>;
}
