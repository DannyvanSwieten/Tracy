import * as React from 'react';
import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Stack';
import Box from '@mui/material/Box';
import Accordion from '@mui/material/Accordion';
import AccordionSummary from '@mui/material/AccordionSummary';
import Typography from '@mui/material/Typography';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import { ListItemButton, ListItemText } from '@mui/material';
import { DndProvider, useDrag } from 'react-dnd'

function DraggableAssetComponent({ id, name, type }) {
    const [{ isDragging }, dragRef] = useDrag({
        type: type,
        item: { id, name },
        collect: (monitor) => ({
            isDragging: monitor.isDragging()
        })
    })

    return (
        <ListItemButton dense key={id} ref={dragRef}>
            <ListItemText>
                <Typography>{name}</Typography>
            </ListItemText>
        </ListItemButton>);
}

function AssetAccordion(props) {

    const style = {
        width: '100%',
        maxWidth: 360,
        bgcolor: 'grey'
    };

    var items = [];
    for (var i = 0; i < props.items.length; i++) {
        items.push(<DraggableAssetComponent key={i} id={i} name={props.items[i]} type={props.type} />)
    }

    return (
        <Stack spacing={0} divider={<Divider orientation="horizontal" flexItem />}>
            <Accordion height={10} sx={{ bgcolor: '#272727' }}>
                <AccordionSummary
                    expandIcon={<ExpandMoreIcon />}
                    aria-controls="panel1a-content"
                    id="panel1a-header"
                >
                    <Typography>{props.name}</Typography>
                </AccordionSummary>

                <List sx={style}>
                    {items}
                </List>
            </Accordion>
        </Stack>
    )
}

export default function AssetBrowser() {
    return (
        <Box component='div' overflow='auto' height='100%'>
            <Stack spacing={0.2} margin={.5}>
                <Typography color='white' textAlign='center'>Assets</Typography>
                <AssetAccordion name='Shapes' items={["Triangle", "Plane", "Cube", "Sphere"]} type='mesh' />
                <AssetAccordion name='Lights' items={["Sun Light", "Point Light", "Spherical Light", "Environment Light"]} type='light' />
                <AssetAccordion name='Materials' items={["Diffuse", "Glass", "Cloth", "PBR"]} type='material'/>
            </Stack>
        </Box>
    );
}