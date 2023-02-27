import React, { FunctionComponent } from 'react';


import {
    StyledButton,
    IconContainer,
} from './styles';
import { Theme } from '../..';

interface StateProps {
    theme: Theme
};

interface OwnProps {
    dark?: boolean
    transparent?: boolean
    width?: number
    content?: string
    icon?: React.ReactNode
    fullWidth?: boolean
    onClick?(): void
}

type Props = OwnProps & StateProps;

const Button: FunctionComponent<Props> = (props) => {
    return (
        <StyledButton
            onClick={props.onClick}
            fullWidth={props.fullWidth}
            transparent={props.transparent}
            dark={props.dark}
        >
            {props.icon && (
                <IconContainer>
                    {props.icon}
                </IconContainer>
            )}
            {props.content}
        </StyledButton>
    );
};

Button.defaultProps = {
    content: 'Button',
    fullWidth: false,
    onClick: () => {},
}

export default Button;