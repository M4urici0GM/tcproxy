
import React from 'react';
import {
  BrowserRouter,
  Route,
  Routes
} from "react-router-dom";


import SignIn from './pages/signin';
import Signup from './pages/signup';
import NotFound from './pages/notfound';

const Router = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/signin" element={<SignIn />} />
        <Route path="/signup" element={<Signup />} />
        <Route path="*" element={<NotFound />} />
      </Routes>
    </BrowserRouter>
  );
};

export default Router;
