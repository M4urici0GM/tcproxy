import React from 'react';

import {
  BackgroundContainer,
  WhiteContainer,
  ContentContainer
} from './styles';
import { Column } from '../Grid';
import LoadingOverlay from '../LoadingOverlay';
import { useAppContext } from '../../contexts/AppContext';

interface Props {
  children: React.ReactNode
}
const AppContainer: React.FC<Props> = (props) => {
  const { loading } = useAppContext();

  return (
    <BackgroundContainer>
      <WhiteContainer>
        <LoadingOverlay loading={loading} />
        <ContentContainer>
          <Column>
            {props.children}
          </Column>
        </ContentContainer>
      </WhiteContainer>
    </BackgroundContainer>
  );
};

export default AppContainer;
