import React, { FC } from 'react';

import { ToastContainer } from 'react-toastify';
import { ThemeProvider } from 'styled-components';


import Router from './router';
import AppContainer from './components/AppContainer';

import 'react-toastify/dist/ReactToastify.css'
import { useAppContext } from './contexts/AppContext';

type Props = {};

const App: FC<Props> = (props) => {
  const { currentTheme } = useAppContext();

  return (
    <ThemeProvider theme={currentTheme}>
      <AppContainer>
        <ToastContainer />
        <Router />
      </AppContainer>
    </ThemeProvider>
  );
};

export default App;


