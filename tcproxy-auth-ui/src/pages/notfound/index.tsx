import React from 'react';
import { UndrawNotFound } from 'react-undraw';

import { Row, Column } from '../../components/Grid';
import { Header } from './styles';

const NotFound: React.FC = () => {

  return (
    <Row>
      <Column sm={12}>
        <UndrawNotFound height="300" />
        <Header>
          Ops, nothing here.
        </Header>
      </Column>
    </Row>
  );
};

export default NotFound;
