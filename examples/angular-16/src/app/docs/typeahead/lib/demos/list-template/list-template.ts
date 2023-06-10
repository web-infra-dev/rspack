import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-typeahead-templates',
  templateUrl: './list-template.html',
  styles: [`
    .custom-list-group {
      display: flex;
      flex-direction: column;
      width: 300px;
      padding-left: 0;
      margin: 0;
      list-style: none;
    }

    .custom-list-group-item {
      position: relative;
      display: block;
      padding: .75rem 1.25rem;
      background-color: #fff;
    }

    .custom-list-group-item.active {
      z-index: 2;
      color: #fff;
      background-color: #FF4461;
      border-color: #FF4461;
    }
  `]
})
export class DemoTypeaheadListTemplateComponent {
  selected?: string;
  states: string[] = [
    'Alabama',
    'Alaska',
    'Arizona',
    'Arkansas',
    'California',
    'Colorado',
    'Connecticut',
    'Delaware',
    'Florida',
    'Georgia',
    'Hawaii',
    'Idaho',
    'Illinois',
    'Indiana',
    'Iowa',
    'Kansas',
    'Kentucky',
    'Louisiana',
    'Maine',
    'Maryland',
    'Massachusetts',
    'Michigan',
    'Minnesota',
    'Mississippi',
    'Missouri',
    'Montana',
    'Nebraska',
    'Nevada',
    'New Hampshire',
    'New Jersey',
    'New Mexico',
    'New York',
    'North Dakota',
    'North Carolina',
    'Ohio',
    'Oklahoma',
    'Oregon',
    'Pennsylvania',
    'Rhode Island',
    'South Carolina',
    'South Dakota',
    'Tennessee',
    'Texas',
    'Utah',
    'Vermont',
    'Virginia',
    'Washington',
    'West Virginia',
    'Wisconsin',
    'Wyoming'
  ];
}
