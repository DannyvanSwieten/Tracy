import { gql } from '@apollo/client';

const CREATE_BASIC_SHAPE = gql`
mutation CreateShape($shape: String!) {
  createBasicShape(shape: $shape) 
} 
`
;

export default CREATE_BASIC_SHAPE; 