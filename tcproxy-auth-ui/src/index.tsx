import React from 'react';
import { StrictMode } from 'react';
import {createRoot} from 'react-dom/client';

import App from './App';
import { AppContextProvider } from './contexts/AppContext';

export interface Theme {
  mode: string
  primaryColor: string
  primaryText: string
  primaryDark: string
  secondaryColor: string
  secondaryText: string
  backgroundColor: string
}

export enum AvailableThemes {
  LIGHT = 'LIGHT',
}

export const defaultTheme: Theme = {
  mode: 'light',
  backgroundColor: '',
  primaryColor: '#5E81AC',
  primaryText: '#3B4252',
  primaryDark: '#3F3D56',
  secondaryColor: '',
  secondaryText: '',
};

const rootElement = document.getElementById('root');
const root = createRoot(rootElement as Element);

root.render(
  <StrictMode>
    <AppContextProvider>
      <App />
    </AppContextProvider>
  </StrictMode>,
);

