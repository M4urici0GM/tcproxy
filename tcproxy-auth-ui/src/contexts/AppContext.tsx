import React from 'react';
import { Theme } from '..';

export interface IAppContext {
  loading: boolean;
  loginHint?: string;
  currentTheme: Theme,
  toggleLoadingState: () => void;
  setLoginHint: (hint: string) => void;
}

const defaultTheme: Theme = {
  mode: 'light',
  backgroundColor: '',
  primaryColor: '#5E81AC',
  primaryText: '#3B4252',
  primaryDark: '#3F3D56',
  secondaryColor: '',
  secondaryText: '',
};

const defaultState: IAppContext = {
  loading: false,
  loginHint: '',
  currentTheme: { ...defaultTheme },
  toggleLoadingState: () => null,
  setLoginHint: () => null,
};

const appContext = React.createContext<IAppContext>({ ...defaultState });

const useAppContext = () => React.useContext(appContext);

const AppContextProvider: React.FC<{ children: React.ReactNode }> = (props) => {
  const { children } = props;
  const [state, setState] = React.useState<IAppContext>({ ...defaultState });

  const setLoginHint = (hint: string) => {
    setState((currentState) => ({ ...currentState, loginHint: hint }));
  };

  const toggleLoadingState = (status = null) => {
    setState(({ loading, ...remaining }) => ({
      ...remaining,
      loading: status ?? !loading,
    }));
  };

  return (
    <appContext.Provider
      value={{
        ...state,
        setLoginHint,
        toggleLoadingState,
      }}
    >
      {children}
    </appContext.Provider>
  );
};

export { AppContextProvider, useAppContext };