import React from 'react';

import {
  StyledInput,
  StyledInputLabel,
  InputContainer,
  IconContainer,
  Icon,
} from './styles';
import {InputProps} from './types';

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

export default Input;
