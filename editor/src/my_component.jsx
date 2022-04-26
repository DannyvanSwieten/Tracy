import { gql, useMutation } from '@apollo/client';

// Define mutation
const NEW_PROJECT = gql`
  # Increments a back-end counter and gets its resulting value
  mutation {
    newProject(name: "Aw Yeah")
  }
`;

export default function MyComponent() {
    // Pass mutation to useMutation
    let input;
    const [addTodo, { data, loading, error }] = useMutation(NEW_PROJECT);

    if (loading) return 'Submitting...';
    if (error) return `Submission error! ${error.message}`;

    return (
        <div className='App-header'>
            <button>Add Todo</button>
        </div>
    );
}