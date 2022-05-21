import Middle from './middle';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import AppBar from './app_bar';
import { Box, Stack } from '@mui/material';
import { ApolloProvider } from '@apollo/client';
import { client } from '../gql/gql_client';
import { DndProvider, useDrag } from 'react-dnd'
import { HTML5Backend } from 'react-dnd-html5-backend'

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
  },
  typography: {
    fontSize: 10,
  },
});

function App() {

  return (
    <ApolloProvider client={client}>
      <ThemeProvider theme={darkTheme}>
        <DndProvider backend={HTML5Backend}>
          <Stack spacing={0.25} height='100vh' width='100%'>
            <AppBar />
            <Middle />
            <Box textAlign='center' bgcolor='#272727' height='30%'>Timeline</Box>
          </Stack>
        </DndProvider>
      </ThemeProvider>
    </ApolloProvider>
  )
}

export default App;
