import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-typeahead-latinize',
  templateUrl: './latinize.html'
})
export class DemoTypeaheadLatinizeComponent {
  selected?: string;
  frenchWords: string[] = [
    'popularisé',
    'français',
    'intéressé',
    'générateur',
    'répandue',
    'répétition',
    'súper'
    ];
}
