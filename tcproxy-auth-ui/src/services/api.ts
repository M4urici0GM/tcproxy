import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:5066',
  timeout: 15000,
  timeoutErrorMessage: 'Timeout limit reached.',
});

export default api;
