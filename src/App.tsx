import { DndContext, DragEndEvent } from '@dnd-kit/core';
import { useState } from 'react';
import { Draggable } from './components/Draggable';
import { Droppable } from './components/Droppable';

export function App() {
  const [isDropped, setIsDropped] = useState(false);
  const draggableMarkup = <Draggable>hello, world</Draggable>;

  const handleDragEnd = (event: DragEndEvent) => {
    if (event.over && event.over.id === 'droppable') {
      setIsDropped(true);
    }
  };

  return (
    <div className="h-full p-4">
      <div className="h-full w-full bg-orange-200 border-orange-300 border">
        <DndContext onDragEnd={handleDragEnd}>
          {!isDropped ? draggableMarkup : null}
          <Droppable>{isDropped ? draggableMarkup : 'Drop here'}</Droppable>
        </DndContext>
      </div>
    </div>
  );
}
