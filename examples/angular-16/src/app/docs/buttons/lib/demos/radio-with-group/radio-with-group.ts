import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-buttons-radio-with-group',
  templateUrl: './radio-with-group.html'
})
export class DemoButtonsRadioWithGroupComponent {
  radioModel = 'Middle';
  radioModelDisabled = 'Middle';
  modelGroupDisabled=false;
}
