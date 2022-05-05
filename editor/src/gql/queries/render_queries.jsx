import { gql } from '@apollo/client';

const RENDER_OUTPUT_QUERY = gql`
query{
  renderer_output
} 
`
;

export default RENDER_OUTPUT_QUERY; 