import { Component } from '@angular/core';
import { AlertComponent } from 'ngx-bootstrap/alert';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-alert-timeout',
  templateUrl: './dismiss-on-timeout.html'
})
export class DemoAlertTimeoutComponent {
  alerts: any[] = [{
    type: 'success',
    msg: `Well done! You successfully read this important alert message. (added: ${new Date().toLocaleTimeString()})`,
    timeout: 5000
  }];

  add(): void {
    this.alerts.push({
      type: 'info',
      msg: `This alert will be closed in 5 seconds (added: ${new Date().toLocaleTimeString()})`,
      timeout: 5000
    });
  }

  onClosed(dismissedAlert: AlertComponent): void {
    this.alerts = this.alerts.filter(alert => alert !== dismissedAlert);
  }
}
