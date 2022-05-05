import { gql } from '@apollo/client';

const RENDER = gql`
mutation Render($batches: Int!) {
  build,
  render(batches: $batches) 
} 
`
;

export default RENDER;