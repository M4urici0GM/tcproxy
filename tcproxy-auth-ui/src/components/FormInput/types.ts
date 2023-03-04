import React from 'react';
import {InputProps} from '@chakra-ui/react';

export interface IFormInputProps extends InputProps {
  name: string;
  showErrors?: boolean;
  type: 'text' | 'email' | 'password',
  disabled?: boolean;
  placeholder?: string;
  fullWidth?: boolean;
  icon?: React.ReactNode;
}