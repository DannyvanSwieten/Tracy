import logo from './logo.svg';
import './App.css';
import { gql, useMutation } from '@apollo/client';

// Define mutation
const NEW_PROJECT = gql`
  mutation {
    newProject(name: "Aw Yeah")
  }
`;

function App() {

  let input;
  //const [addTodo, { data, loading, error }] = useMutation(NEW_PROJECT);

  //if (loading) return 'Submitting...';
  //if (error) return `Submission error! ${error.message}`;

  //addTodo();

  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>
          Edit <code>src/App.js</code> and save to reload.
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );
}

export default App;
