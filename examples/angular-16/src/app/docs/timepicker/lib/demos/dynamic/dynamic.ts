import { Component } from '@angular/core';
import { TimepickerConfig } from 'ngx-bootstrap/timepicker';

export function getTimepickerConfig(): TimepickerConfig {
  return Object.assign(new TimepickerConfig(), {
    allowEmptyTime: true
  });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-dynamic',
  templateUrl: './dynamic.html',
  providers: [{ provide: TimepickerConfig, useFactory: getTimepickerConfig }]
})
export class DemoTimepickerDynamicComponent {
  mytime: Date | undefined = new Date();
  isValid?: boolean;

  update(): void {
    const time = new Date();
    time.setHours(14);
    time.setMinutes(0);

    this.mytime = time;
  }

  changed(): void {
    console.log(`Time changed to: ${this.mytime}`);
  }

  clear(): void {
    this.mytime = void 0;
  }
}
