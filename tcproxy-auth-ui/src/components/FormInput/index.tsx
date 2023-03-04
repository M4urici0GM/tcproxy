import React from 'react';
import {useFormContext} from 'react-hook-form';
import {IFormInputProps} from './types';
import {InputProps, Input, Text, Stack} from '@chakra-ui/react';

const FormInput: React.FC<IFormInputProps> = (props) => {
  const {name, disabled, placeholder} = props;
  const {register, setValue, formState} = useFormContext();
  const { errors } = formState;
  const {
    name: fieldName,
    onBlur,
  } = register(name);

  const containErrors = fieldName in errors;

  return (
    <Stack direction="column">
      <Input
        id={fieldName}
        {...(props as InputProps)}
        onBlur={onBlur}
        disabled={disabled}
        placeholder={placeholder}
        onChange={(e) => setValue(fieldName, e.target.value)}
        isInvalid={containErrors}
        errorBorderColor='red.300'
      />
      {containErrors && (
        <Text
          alignSelf='flex-start'
          color="red.300"
          fontSize="xs"
          m={0}
          margin={0}
          padding={0}
        >
          {errors[fieldName]?.message?.toString() ?? null}
        </Text>
      )}
    </Stack>
  );
};

export default FormInput;
