import React, {FunctionComponent} from "react";
import {Column, Container, Row} from "../../../components/Grid";
import {UndrawSignin} from "react-undraw";
import {FaEnvelope, FaEye, FaSignInAlt} from "react-icons/fa";
import Button from "../../../components/Button";
import {defaultTheme} from "../../../index";
import {SignInButton} from "../styles";
import FormInput from "../../../components/FormInput";

const SignUpForm: FunctionComponent = () => {
  return (
    <Container>
      <Row className="d-flex align-items-center">
        <Column sm={12} md={6}>
          <UndrawSignin height="300" />
        </Column>
        <Column sm={12} md={6}>
          <Row>
            <Column sm={12} md={6} className="mt-3 pr-">
              <FormInput
                fullWidth
                name="firstName"
                label="Your first name"
                type="text"
                placeholder="John"
              />
            </Column>
            <Column sm={12} md={6} className="mt-3">
              <FormInput
                fullWidth
                name="lastName"
                label="Your last name"
                type="text"
                placeholder="Doe"
              />
            </Column>
          </Row>
          <Row className="mt-3">
            <Column>
              <FormInput
                name="email"
                label="Your best email"
                type="email"
                placeholder="your@email.com"
                icon={<FaEnvelope />}
              />
            </Column>
          </Row>
          <Row className="mt-3">
            <Column>
              <FormInput
                name="password"
                label="Your password"
                type="password"
                placeholder="*******"
                icon={<FaEye />}
              />
            </Column>
          </Row>
          <Row className="mt-3">
            <Column>
              <FormInput
                name="passwordConfirm"
                label="Confirm your password"
                type="password"
                placeholder="*******"
              />
            </Column>
          </Row>
          <Row className="mt-3">
            <Column>
              <Button
                type="submit"
                content="SignIn"
                dark
                fullWidth
                theme={defaultTheme}
                icon={<FaSignInAlt />}
              />
            </Column>
          </Row>
          <Row className="d-flex justify-content-center">
            <Column className="d-flex">
              <SignInButton>
                Already have an account? Click here
              </SignInButton>
            </Column>
          </Row>
        </Column>
      </Row>
    </Container>
  );
};

export default SignUpForm;
