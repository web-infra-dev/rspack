import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-mousewheel',
  templateUrl: './mousewheel.html'
})
export class DemoTimepickerMousewheelComponent {
  allowMouseWheel = true;
  myTime = new Date();
}
