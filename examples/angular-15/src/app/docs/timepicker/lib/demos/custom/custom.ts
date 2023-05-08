import { Component } from '@angular/core';

interface IOptions {
  hstep: number[];
  mstep: number[];
  sstep: number[];
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-custom',
  templateUrl: './custom.html'
})
export class DemoTimepickerCustomComponent {
  hstep = 1;
  mstep = 15;
  sstep = 10;

  mytime: Date = new Date();
  options: IOptions = {
    hstep: [1, 2, 3],
    mstep: [1, 5, 10, 15, 25, 30],
    sstep: [5, 10, 20, 30]
  };
}
