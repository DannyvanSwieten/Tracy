import * as React from 'react';
import Box from '@mui/material/Box';
import { useDrop } from 'react-dnd';
import { useMutation } from '@apollo/client';
import CREATE_BASIC_SHAPE from '../gql/mutations/import_mutations';
import RENDER from '../gql/mutations/render';

export default function ViewPort(){

    const [createBasicShape, {data, loading, error}] = useMutation(CREATE_BASIC_SHAPE);
    const [render, {renderData, renderLoading, renderError}] = useMutation(RENDER);
    function doIt(item){
        createBasicShape({ variables: { shape: item.name } });
        render({ variables: { batches: 8 } })
    }
    const [{ isOver }, dropRef] = useDrop({
        accept: 'mesh',
        drop: (item) => 
            doIt(item)
        ,
        collect: (monitor) => ({
            isOver: monitor.isOver()
        })
    })

    console.log(data);

    if(renderData){
        return(
            <Box ref={dropRef} textAlign='center' width='70%' bgcolor='#272727'>Render finished</Box>
        )
    }

    if(renderLoading){
        return(
            <Box ref={dropRef} textAlign='center' width='70%' bgcolor='#272727'>Loading...</Box>
        )
    }

    return(
        <Box ref={dropRef} textAlign='center' width='70%' bgcolor='#272727'>Viewport</Box>
    )
}