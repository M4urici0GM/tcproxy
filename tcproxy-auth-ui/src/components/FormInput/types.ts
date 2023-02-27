import React from "react";

export interface IFormInputProps {
  name: string;
  type: "text" | "email" | "password",
  label: string;
  placeholder?: string;
  fullWidth?: boolean;
  icon?: React.ReactNode;
}