import {
    ApolloClient,
    HttpLink,
    InMemoryCache
} from '@apollo/client';

const httpLink = new HttpLink({
    uri: "http://localhost:7000", 
  })
export const client = new ApolloClient({
    link: httpLink,
    cache: new InMemoryCache()
});

