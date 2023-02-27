import React, { FunctionComponent } from 'react';
import { toast } from 'react-toastify';


import { FormProvider, useForm } from 'react-hook-form';
import {useAppContext} from '../../contexts/AppContext';
import {IFormValues} from './types';
import SignUpForm from './Components/SignUpForm';


const SignUp: FunctionComponent = () => {
  const { toggleLoadingState } = useAppContext();
  const form = useForm<IFormValues>({
    defaultValues: {
      firstName: '',
      lastName: '',
      email: '',
      password: '',
      passwordConfirm: '',
    }
  });

  const onFormSubmit = async (values: IFormValues) => {
    toggleLoadingState();
    console.log('form was submuitted! ', { ...values });
    setTimeout(() => {
      toast.success('Account created successfully, you may login now.', { delay: 0 });
      toggleLoadingState();
    }, 500);
  };

  return (
    <form onSubmit={form.handleSubmit(onFormSubmit)}>
      <FormProvider {...form}>
        <SignUpForm />
      </FormProvider>
    </form>
  );
};

export default SignUp;
