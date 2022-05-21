import { gql } from '@apollo/client';

const NODE_ADDED_SUBSCRIPTION = gql`
  subscription nodeAdded {
    nodeAdded {
      name
    mesh{
      name
    }
    }
  }
`;

export default NODE_ADDED_SUBSCRIPTION;