export class SharedConfig {
  constructor() {
    this.settings = {
      debug: false,
      version: '1.0.0',
      features: ['webgl', 'audio', 'vr']
    };
  }
  
  get(key) {
    return key ? this.settings[key] : this.settings;
  }
}
