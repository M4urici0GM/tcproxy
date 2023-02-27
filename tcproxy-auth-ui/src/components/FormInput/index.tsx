import React from "react";
import {useFormContext} from "react-hook-form";
import Input from "../Input";
import {IFormInputProps} from "./types";

const FormInput: React.FC<IFormInputProps> = (props) => {
  const {
    name,
    type,
    label,
    placeholder,
    fullWidth,
  } = props;
  const { register, setValue } = useFormContext();

  const {
    name: fieldName,
    onBlur,
    disabled
  } = register(name);

  return (
    <Input
      id={fieldName}
      fullWidth={fullWidth}
      label={label}
      type={type}
      // TODO: check why native onChange doesnt work.
      onChange={async (e) => setValue(fieldName, e.target.value)}
      onBlur={onBlur}
      disabled={disabled}
      placeholder={placeholder}
    />
  );
};

export default FormInput;
