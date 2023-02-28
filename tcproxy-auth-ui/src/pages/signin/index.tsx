/* eslint-disable @typescript-eslint/no-unused-vars */
import React, {useState} from 'react';
import {
  Avatar,
  ChakraProvider,
  Container,
  Heading,
  HStack, Progress,
  Stack,
  Tag,
  TagLabel,
  TagRightIcon,
  Text,
  VStack
} from '@chakra-ui/react';
import {Input, Divider, Button, Fade} from '@chakra-ui/react';
import {FcGoogle, FcDown} from 'react-icons/fc';
import { FaChevronDown } from 'react-icons/fa';


import {useAppContext} from '../../contexts/AppContext';
import styled from 'styled-components';
import { CloseIcon } from '@chakra-ui/icons';


const Page = styled.div`
  width: 100%;
  height: 100%;
  padding-top: 6rem;
  padding-bottom: 6rem;
  display: flex;
  background: #dedede;
`;

const PageContainer = styled(Container)`
  background-color: #fff;
  border-radius: 10px;
`;


const SignIn: React.FC = () => {
  const [isPassShown, setIsPassShown] = useState(false);
  const [step, setStep] = useState('EMAIL');
  const [email, setEmail] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState({ has: false, msg: 'Invalid username or password' });

  const {toggleLoadingState} = useAppContext();


  return (
    <ChakraProvider>
      <Page>
        <PageContainer
          paddingY={8}
          paddingX={10}
          boxShadow="lg"
          alignSelf="center"
        >
          <Stack spacing={6}>
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
                Log in into your account
              </Heading>
            </Stack>
            {step == 'EMAIL' && (
              <Fade in>
                <Stack spacing={6}>
                  <Stack spacing={6}>
                    <Button
                      leftIcon={<FcGoogle fontSize={20}/>}
                      fontSize="md"
                      variant="outline"
                    >
                      Continue with google
                    </Button>
                    <Divider/>
                    <Stack spacing={4}>
                      <Stack>
                        <Input
                          placeholder="Your email"
                          value={email}
                          disabled={loading}
                          onChange={(e) => setEmail(e.target.value)}
                          isInvalid={error.has}
                          errorBorderColor='red.300'
                        />
                        {error.has && (
                          <Text
                            alignSelf='center'
                            color="red.300"
                            size="xs"
                          >
                            {error.msg}
                          </Text>
                        )}
                      </Stack>
                      <Button
                        colorScheme='blue'
                        variant='solid'
                        isDisabled={loading}
                        onClick={() => {
                          setLoading(true);
                          setTimeout(() => {
                            setLoading(false);
                            setStep('PASSWORD');
                            // setError((currentState) => ({ has: true, msg: 'Email not registered' }));
                          }, 700);
                        }}
                      >
                        Continue with email
                      </Button>
                      {loading && (
                        <Progress size='xs' isIndeterminate />
                      )}
                    </Stack>
                  </Stack>
                  <Stack alignItems="center">
                    <Text>{'Don\'t have an account yet?'}</Text>
                    <Button
                      variant="link"
                      lineHeight="normal"
                      fontSize="sm"
                    >
                      Create your account
                    </Button>
                  </Stack>
                </Stack>
              </Fade>
            )}
            {step == 'PASSWORD' && (
              <Fade in>
                <Stack spacing={6}>
                  <Stack alignItems="center">
                    <Heading
                      size="sm"
                      fontWeight="semibold"
                    >
                      Hi Mauricio,
                    </Heading>
                    <Tag
                      size="lg"
                      variant='outline'
                      colorScheme='blue'
                      cursor="pointer"
                      borderRadius="full"
                      onClick={() => {
                        setStep('EMAIL');
                        setEmail('');
                      }}
                    >
                      <Avatar
                        src='https://lh3.googleusercontent.com/a/AGNmyxZKbThqOHtmqZAavm5lG3-6KP7fUpg6cqhDbd7cMrc'
                        size='xs'
                        name='Mauricio Barbosa'
                        ml={-1}
                        mr={2}
                      />
                      <TagLabel marginTop={-1}>{email}</TagLabel>
                      <TagRightIcon as={FaChevronDown} />
                    </Tag>
                  </Stack>
                  <Input placeholder="Password"/>
                  <HStack justifyContent="space-between">
                    <Button
                      variant="link"
                      lineHeight="normal"
                      fontSize="sm"
                    >
                      Forgot your password?
                    </Button>
                    <Button disabled colorScheme='blue' variant='solid' alignSelf="flex-end">
                      Log In
                    </Button>
                  </HStack>
                </Stack>
              </Fade>
            )}
          </Stack>
        </PageContainer>
      </Page>
    </ChakraProvider>
  );
};

export default SignIn;
