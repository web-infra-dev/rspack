import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-meridian',
  templateUrl: './meridian.html'
})
export class DemoTimepickerMeridianComponent {
  ismeridian = true;

  mytime: Date = new Date();

  toggleMode(): void {
    this.ismeridian = !this.ismeridian;
  }
}
