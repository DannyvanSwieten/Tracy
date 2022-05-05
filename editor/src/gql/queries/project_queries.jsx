import { gql } from '@apollo/client';

const PROJECT_QUERY = gql`
query{
  project{
    name
  } 
} 
`
;

export default PROJECT_QUERY; 