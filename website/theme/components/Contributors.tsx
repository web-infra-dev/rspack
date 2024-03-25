import { FC } from 'react';

export const Contributors: FC = () => (
  <>
    <hr />
    <div className="flex flex-col my-4 items-center overflow-x-auto">
      <h2 className="text-3xl mt-12 mb-12 font-bold">Contributors</h2>
      <object data="https://opencollective.com/rspack/contributors.svg?width=900&button=false" />
    </div>
  </>
);
