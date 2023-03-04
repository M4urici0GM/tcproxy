import React, {FC} from 'react';

import {ToastContainer} from 'react-toastify';
import {ThemeProvider} from 'styled-components';
import {ChakraProvider} from '@chakra-ui/react';

import Router from './router';
import 'react-toastify/dist/ReactToastify.css';
import {useAppContext} from './contexts/AppContext';

// eslint-disable-next-line @typescript-eslint/ban-types
type Props = {};
const App: FC<Props> = () => {
  const {currentTheme} = useAppContext();

  return (
    <ChakraProvider>
      <ThemeProvider theme={currentTheme}>
        <ToastContainer/>
        <Router/>
      </ThemeProvider>
    </ChakraProvider>
  );
};

export default App;


