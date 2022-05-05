import * as React from 'react';
import Stack from '@mui/material/Stack';
import AssetBrowser from './assets_browser';
import ViewPort from './viewport';
import { Box } from '@mui/system';
import PROJECT_QUERY from '../gql/queries/project_queries';
import { ApolloClient, useQuery } from '@apollo/client';
import Typography from '@mui/material/Typography';
import Modal from '@mui/material/Modal';


export default function Middle(props) {
    const { loading, error, data } = useQuery(PROJECT_QUERY);
    console.log(loading);
    console.log(error);
    const [open, setOpen] = React.useState(true);
    if (loading) {
        return (
            <div>Loading...</div>
        )
    }

    if (error) {
        const handleClose = () => setOpen(false);
        const style = {
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            width: 400,
            bgcolor: 'background.paper',
            border: '2px solid #000',
            boxShadow: 24,
            p: 4,
            color: 'white'
        };

        return (
            <Modal
                open={true}
                onClose={handleClose}
                aria-labelledby="modal-modal-title"
                aria-describedby="modal-modal-description"
            >
                <Box sx={style}>
                    <Typography id="modal-modal-title" variant="h6" component="h2">
                        {error.message}
                    </Typography>
                </Box>
            </Modal>
        )
    }

    return (
        <Stack
            direction="row" height='80%' spacing={0.25}
        >
            <Box bgcolor='#272727' width='1.5%' height='100%'></Box>
            <Box margin={0.1} bgcolor='#272727' width='10%' height='100%'>
                <AssetBrowser />
            </Box>
            <ViewPort />
            <Box textAlign='center' width='20%' height='100%' bgcolor='#272727'>{data.project.name}</Box>
        </Stack>
    )
}