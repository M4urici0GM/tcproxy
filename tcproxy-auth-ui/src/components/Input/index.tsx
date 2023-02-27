import React, { ChangeEvent, FocusEvent } from 'react';

import {
  StyledInput,
  StyledInputLabel,
  InputContainer,
  IconContainer,
  Icon,
} from './styles';

interface InputProps {
  fullWidth?: boolean
  type?: "text" | "email" | "password"
  width?: number
  placeholder?: string
  label?: string
  icon?: React.ReactNode
  iconColor?: string
  style?: any;
  id?: string;
  disabled?: boolean;
  onBlur?: ((event: FocusEvent<HTMLInputElement>) => Promise<void | boolean>) | (() => void);
  onFocus?: ((event: FocusEvent<HTMLInputElement>) => Promise<void | boolean>) | (() => void);
  onChange?: ((event: ChangeEvent<HTMLInputElement>) => Promise<void | boolean>) | (() => void);
}

const Input: React.FC<InputProps> = (props) => {
  return (
    <InputContainer
      fullWidth={props.fullWidth}
      width={props.width}
      style={props.style}
    >
      {props.label && (
        <StyledInputLabel>
          {props.label}
        </StyledInputLabel>
      )
      }
      <IconContainer>
        <StyledInput
          id={props.id}
          type={props.type}
          onChange={props.onChange}
          onFocus={props.onFocus}
          onBlur={props.onBlur}
          disabled={props.disabled}
          placeholder={props.placeholder}
        />
        {props.icon && (
          <Icon>
            {props.icon}
          </Icon>
        )}
      </IconContainer>
    </InputContainer>
  );
};

Input.defaultProps = {
  fullWidth: true,
  type: 'text',
  placeholder: '',
  style: {},
  onBlur: () => { },
  onFocus: () => { },
  onChange: () => { },
};

export default Input;
