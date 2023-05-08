import { Component, OnInit } from '@angular/core';

import { TypeaheadOrder } from 'ngx-bootstrap/typeahead';
import { Observable, of, Subscriber } from 'rxjs';
import { switchMap } from 'rxjs/operators';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-typeahead-ordering',
  templateUrl: './ordering.html'
})
export class DemoTypeaheadOrderingComponent implements OnInit {
  selected1?: string;
  selected2?: string;
  selected3?: string;
  selected4?: string;
  sortConfig1: TypeaheadOrder = {
    direction: 'desc'
  };
  sortConfig2: TypeaheadOrder = {
    direction: 'asc'
  };
  sortConfig3: TypeaheadOrder = {
    direction: 'asc',
    field: 'city'
  };
  states$?: Observable<string[]>;
  states: string[] = [
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
    'Alaska',
    'Alabama',
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
    'Vermont',
    'Virginia',
    'Washington',
    'West Virginia',
    'Wisconsin',
    'Wyoming'
  ];
  cities = [{
    city: 'Norton',
    state: 'Virginia',
    code: '61523'
  }, {
    city: 'Grundy',
    state: 'Virginia',
    code: '77054'
  }, {
    city: 'Coeburn',
    state: 'Virginia',
    code: '01665'
  }, {
    city: 'Phoenix',
    state: 'Arizona',
    code: '29128'
  }, {
    city: 'Tucson',
    state: 'Arizona',
    code: '32084'
  }, {
    city: 'Mesa',
    state: 'Arizona',
    code: '21465'
  }, {
    city: 'Independence',
    state: 'Missouri',
    code: '26887'
  }, {
    city: 'Kansas City',
    state: 'Missouri',
    code: '79286'
  }, {
    city: 'Springfield',
    state: 'Missouri',
    code: '92325'
  }, {
    city: 'St. Louis',
    state: 'Missouri',
    code: '64891'
  }];

  ngOnInit(): void {
    this.states$ = new Observable((observer: Subscriber<string>) => {
      // Runs on every search
      observer.next(this.selected4);
    })
      .pipe(
        switchMap((token: string) => {
          const query = new RegExp(token, 'i');

          return of(
            this.states.filter((state: string) => query.test(state))
          );
        })
      );
  }
}
