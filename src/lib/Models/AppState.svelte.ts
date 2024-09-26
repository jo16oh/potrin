type AppState = {
  name: string;
};

export class App {
  private static _instance: App | undefined = $state();
  private _state: AppState;

  private constructor(state: AppState) {
    this._state = state;
  }

  static async init(state: AppState) {
    this._instance = new App(state);
  }

  static get state() {
    if (!this._instance)
      throw new Error("App is not initialized. Call App.init() first.");
    return this._instance._state;
  }
}
