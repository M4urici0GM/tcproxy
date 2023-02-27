import React, {ChangeEvent, FocusEvent} from "react";

export interface InputProps {
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