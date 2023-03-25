import React, {FunctionComponent, useState} from 'react';
import { AxiosError } from 'axios';
import {useForm, FormProvider} from 'react-hook-form';
import {Button, Fade, Heading, Progress, Stack} from '@chakra-ui/react';
import {useNavigate} from 'react-router-dom';
import { toast } from 'react-toastify';


import {IFormValues} from './types';
import FormInput from '../../components/FormInput';
import {Page, PageContainer, Form} from './styles';
import axiosApi from '../../services/api';
import {useAppContext} from '../../contexts/AppContext';

interface ApiError {
  statusCode: number,
  errors: Array<{ property: string; message: string;}>;
}

interface User {
  id: string;
  name: string;
  email: string;
}

interface CreateUserRequest {
  firstName: string;
  lastName: string;
  email: string;
  password: string;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const createUser = async (user: CreateUserRequest) => {
  return axiosApi.post<CreateUserRequest, User>('/v1/user', {...user});
};


const SignUp: FunctionComponent = () => {
  const navigate = useNavigate();

  const {
    setLoginHint,
  } = useAppContext();
  const [loading] = useState(false);
  const form = useForm<IFormValues>({
    defaultValues: {
      firstName: '',
      lastName: '',
      email: '',
      password: '',
      passwordConfirm: '',
    }
  });

  const onGoBackButtonClick = () => navigate('/signin');

  const onFormSubmit = async (values: IFormValues) => {
    console.log('form was submitted! ', {...values});
    const { firstName, lastName, email, password, passwordConfirm } = values;
    if (password !== passwordConfirm) {
      form.setError('passwordConfirm', { message: 'Passwords doesn\'t match' });
      return;
    }

    try {
      const response = await createUser({
        firstName,
        lastName,
        password,
        email,
      });

      setLoginHint(response.email);
    } catch (err) {
      const axiosError = err as AxiosError<ApiError>;
      if (axiosError.response?.status === 400) {
        return null;
      }

      toast.error('There was an error trying to create your user, try again later..');
    }
  };

  return (
    <Page>
      <PageContainer
        paddingY={8}
        paddingX={10}
        boxShadow="lg"
        alignSelf="center"
      >
        <Fade in>
          <FormProvider {...form}>
            <Form onSubmit={form.handleSubmit(onFormSubmit)}>
              <Stack spacing={8}>
                <Stack alignItems="center">
                  <Heading
                    size="lg"
                    fontWeight="semibold"
                  >
                    Tcproxy
                  </Heading>
                  <Heading
                    size="xs"
                    fontWeight="semibold"
                  >
                    Create your account
                  </Heading>
                </Stack>
                <Stack direction="row">
                  <FormInput
                    type="text"
                    disabled={loading}
                    name="firstName"
                    placeholder="First Name"
                  />
                  <FormInput
                    type="text"
                    disabled={loading}
                    name="lastName"
                    placeholder="Last Name"
                  />
                </Stack>
                <Stack>
                  <FormInput
                    type="email"
                    name="email"
                    disabled={loading}
                    placeholder="Your email"
                  />
                </Stack>
                <Stack>
                  <FormInput
                    type="password"
                    disabled={loading}
                    name="password"
                    placeholder="Password"
                  />
                </Stack>
                <Stack>
                  <FormInput
                    type="password"
                    name="passwordConfirm"
                    disabled={loading}
                    placeholder="Confirm Your password"
                  />
                </Stack>
                {loading && (
                  <Progress size='xs' isIndeterminate/>
                )}
                <Stack spacing={6} justifyContent="space-between">
                  <Button
                    isDisabled={loading}
                    colorScheme='blue'
                    variant='solid'
                    type="submit"
                  >
                    Register
                  </Button>
                  <Button
                    variant="link"
                    lineHeight="normal"
                    fontSize="sm"
                    onClick={onGoBackButtonClick}
                    isDisabled={loading}
                  >
                    Back
                  </Button>
                </Stack>
              </Stack>
            </Form>
          </FormProvider>
        </Fade>
      </PageContainer>
    </Page>
  );
};

export default SignUp;
