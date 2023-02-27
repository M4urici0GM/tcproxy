import {Theme} from "../.."

export interface IAppState {
  readonly themeMode: Theme
  readonly loading: boolean
  readonly error: false
}

export interface IStateProps {
  app: IAppState
}

export interface IDispatchProps {
  toggleLoadingStatus(): void
};
